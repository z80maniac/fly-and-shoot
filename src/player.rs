// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

use crate::anim::{AnimationTimer, AssetsLoading};
use crate::bullet::{Bullet, BulletRes, BulletTimer, HitBox};
use crate::collision::Screen;
use crate::enemy::{Enemy, EnemyBullet};
use crate::explosion::{Explosion, ExplosionAudio, ExplosionSheet};
use crate::{AnimPlugin, GameState};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;

#[cfg(feature = "inspector")]
use bevy_inspector_egui::Inspectable;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system_set(SystemSet::on_enter(GameState::PlayerSlideOut).with_system(spawn))
            .add_system_set(SystemSet::on_update(GameState::PlayerSlideOut).with_system(slide_out))
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(movement)
                    .with_system(attack)
                    .with_system(collision_with_enemy)
                    .with_system(collision_with_bullet),
            );
    }
}

const WIDTH: f32 = 152.0;
const HEIGHT: f32 = 83.0;
const SIZE_X: f32 = 0.15;
const SIZE_Y: f32 = SIZE_X * HEIGHT / WIDTH;
const SPEED: f32 = 1.0;
const SLIDE_OUT_SLOWDOWN: f32 = 5.0;
const BULLET_SPEED: f32 = 2.0;
const SPEED_CHANGE: f32 = 4.5;

const EXHAUST_WIDTH: f32 = 75.0;
const EXHAUST_HEIGHT: f32 = 25.0;
const EXHAUST_SIZE_X: f32 = 0.1;
const EXHAUST_SIZE_Y: f32 = EXHAUST_SIZE_X * EXHAUST_HEIGHT / EXHAUST_WIDTH;

#[derive(Component)]
#[cfg_attr(feature = "inspector", derive(Inspectable))]
pub struct Player {
    cur_speed_vec: Vec2,
    heat: f32,
    heat_recovery: f32,
}

impl Player {
    const BULLET_HEAT: f32 = 0.05;
    const MIN_HEAT_RECOVERY: f32 = 0.25;
    const MAX_HEAT_RECOVERY: f32 = 0.5;
    const HEAT_RECOVERY_INCREASE: f32 = 0.1;

    fn new() -> Self {
        return Self {
            cur_speed_vec: Vec2::ZERO,
            heat: 0.0,
            heat_recovery: 0.0,
        };
    }

    fn increase_heat(&mut self) {
        self.heat += Self::BULLET_HEAT;
        self.heat_recovery = Self::MIN_HEAT_RECOVERY;
    }

    fn cooldown(&mut self, delta: f32) {
        self.heat_recovery = (self.heat_recovery + Self::HEAT_RECOVERY_INCREASE * delta)
            .min(Self::MAX_HEAT_RECOVERY);
        self.heat = (self.heat - self.heat_recovery * delta).max(0.0);
    }
}

#[derive(Component)]
pub struct PlayerBullet;

pub struct PlayerGraphics {
    ship_atlas: Handle<TextureAtlas>,
    exhaust_atlas: Handle<TextureAtlas>,
}

fn spawn(mut commands: Commands, player_graphics: Res<PlayerGraphics>) {
    let mut sprite = TextureAtlasSprite::new(0);
    sprite.custom_size = Some(Vec2::new(SIZE_X, SIZE_Y));
    let player = commands
        .spawn_bundle(SpriteSheetBundle {
            sprite,
            texture_atlas: player_graphics.ship_atlas.clone(),
            transform: Transform {
                translation: Vec3::new(-SIZE_X, 0.5 + SIZE_Y / 2.0, 200.0),
                ..default()
            },
            ..default()
        })
        .insert(Player::new())
        .insert(BulletTimer::new(0.1))
        .insert(HitBox(Vec2::new(SIZE_X * 0.9, SIZE_Y * 0.9)))
        .insert(Name::new("Player"))
        .id();

    let mut sprite = TextureAtlasSprite::new(0);
    sprite.custom_size = Some(Vec2::new(EXHAUST_SIZE_X, EXHAUST_SIZE_Y));
    let exhaust = commands
        .spawn_bundle(SpriteSheetBundle {
            sprite,
            texture_atlas: player_graphics.exhaust_atlas.clone(),
            transform: Transform {
                translation: Vec3::new(-0.11, -0.015, -1.0),
                ..default()
            },
            ..default()
        })
        .insert(AnimationTimer::infinite(0.1))
        .insert(Name::new("PlayerExhaust"))
        .id();

    commands.entity(player).add_child(exhaust);
}

fn load_assets(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut loading: ResMut<AssetsLoading>,
) {
    let ship_atlas = AnimPlugin::load_atlas(
        "player.png",
        Vec2::new(WIDTH, HEIGHT),
        1,
        1,
        &assets,
        &mut atlases,
        &mut loading,
    );
    let exhaust_atlas = AnimPlugin::load_atlas(
        "player_exhaust.png",
        Vec2::new(EXHAUST_WIDTH, EXHAUST_HEIGHT),
        2,
        2,
        &assets,
        &mut atlases,
        &mut loading,
    );

    commands.insert_resource(PlayerGraphics {
        ship_atlas,
        exhaust_atlas,
    });
}

fn movement(
    mut q: Query<(&mut Transform, &mut Player)>,
    kbd: Res<Input<KeyCode>>,
    time: Res<Time>,
    win: Res<WindowDescriptor>,
) {
    let bounds = win.bounds_box_inside(Vec2::new(SIZE_X, SIZE_Y));

    let speed = if kbd.pressed(KeyCode::LShift) {
        SPEED / 2.0
    } else {
        SPEED
    };

    let mut target_speed_vector = Vec2::ZERO;

    for (mut transform, mut player) in q.iter_mut() {
        if kbd.pressed(KeyCode::A) {
            target_speed_vector -= Vec2::X;
        }
        if kbd.pressed(KeyCode::D) {
            target_speed_vector += Vec2::X;
        }
        if kbd.pressed(KeyCode::W) {
            target_speed_vector += Vec2::Y;
        }
        if kbd.pressed(KeyCode::S) {
            target_speed_vector -= Vec2::Y;
        }

        target_speed_vector = target_speed_vector.normalize_or_zero();
        let speed_change = target_speed_vector - player.cur_speed_vec;
        let speed_change_norm = speed_change.normalize_or_zero();
        if speed_change_norm == Vec2::ZERO {
            player.cur_speed_vec = target_speed_vector;
        } else {
            let speed_change_dist = speed_change.length();
            let frame_speed_change = SPEED_CHANGE * time.delta_seconds();
            if speed_change_dist < frame_speed_change {
                player.cur_speed_vec = target_speed_vector;
            } else {
                player.cur_speed_vec += speed_change_norm * frame_speed_change;
            }
        }

        if player.cur_speed_vec != Vec2::ZERO {
            let old_pos = Vec2::new(transform.translation.x, transform.translation.y);
            let pos_delta = speed * time.delta_seconds() * player.cur_speed_vec;
            let mut next_pos = old_pos + pos_delta;

            if next_pos.x < bounds.left {
                next_pos.x = bounds.left;
                player.cur_speed_vec.x = 0.0;
            } else if next_pos.x > bounds.right {
                next_pos.x = bounds.right;
                player.cur_speed_vec.x = 0.0;
            }

            if next_pos.y < bounds.bottom {
                next_pos.y = bounds.bottom;
                player.cur_speed_vec.y = 0.0;
            } else if next_pos.y > bounds.top {
                next_pos.y = bounds.top;
                player.cur_speed_vec.y = 0.0;
            }

            transform.translation = next_pos.extend(transform.translation.z);
        }
    }
}

fn attack(
    mut commands: Commands,
    mut q: Query<(&Transform, &mut BulletTimer, &mut Player)>,
    kbd: Res<Input<KeyCode>>,
    bullet_res: Res<BulletRes>,
    audio: Res<Audio>,
    time: Res<Time>,
) {
    for (transform, mut bullet_timer, mut player) in q.iter_mut() {
        bullet_timer.process(time.delta());

        if kbd.pressed(KeyCode::M) && bullet_timer.can_shoot && player.heat < 1.0 {
            let starting_point = transform.translation + Vec3::new(0.03, -0.025, 1.0);
            let mut color = Color::WHITE;
            color.set_b(1.0 - player.heat);
            let entity = Bullet::spawn(
                &mut commands,
                &bullet_res.player,
                starting_point,
                starting_point + Vec3::X,
                BULLET_SPEED,
                color,
                &audio,
            );
            commands
                .entity(entity)
                .insert(PlayerBullet)
                .insert(Name::new("PlayerBullet"));
            bullet_timer.shoot();
            player.increase_heat();
        } else {
            player.cooldown(time.delta_seconds());
        }
    }
}

fn collision_with_enemy(
    mut commands: Commands,
    player_query: Query<(&Transform, &HitBox, Entity), With<Player>>,
    enemy_query: Query<(&Transform, &HitBox, Entity), With<Enemy>>,
    explosion_sheet: Res<ExplosionSheet>,
    mut game_state: ResMut<State<GameState>>,
    explosion_audio: Res<ExplosionAudio>,
    audio: Res<Audio>,
) {
    for (player_pos, player_box, player) in player_query.iter() {
        for (enemy_pos, enemy_box, enemy) in enemy_query.iter() {
            if collide(
                player_pos.translation,
                player_box.0,
                enemy_pos.translation,
                enemy_box.0,
            )
            .is_some()
            {
                commands.entity(player).despawn_recursive();
                commands.entity(enemy).despawn_recursive();

                Explosion::spawn(
                    &mut commands,
                    &explosion_sheet,
                    player_pos.translation,
                    0.05,
                    &explosion_audio,
                    &audio,
                );
                Explosion::spawn(
                    &mut commands,
                    &explosion_sheet,
                    enemy_pos.translation,
                    0.05,
                    &explosion_audio,
                    &audio,
                );

                game_state.set(GameState::GameOver).unwrap();
                return;
            }
        }
    }
}

fn collision_with_bullet(
    mut commands: Commands,
    player_query: Query<(Entity, &Transform, &HitBox), With<Player>>,
    enemy_bullet_query: Query<(Entity, &Transform, &HitBox), With<EnemyBullet>>,
    explosion_sheet: Res<ExplosionSheet>,
    explosion_audio: Res<ExplosionAudio>,
    audio: Res<Audio>,
    mut game_state: ResMut<State<GameState>>,
) {
    for (player, player_pos, enemy_hitbox) in player_query.iter() {
        for (bullet, bullet_pos, bullet_hitbox) in enemy_bullet_query.iter() {
            if collide(
                player_pos.translation,
                enemy_hitbox.0,
                bullet_pos.translation,
                bullet_hitbox.0,
            )
            .is_some()
            {
                commands.entity(player).despawn_recursive();
                commands.entity(bullet).despawn_recursive();

                Explosion::spawn(
                    &mut commands,
                    &explosion_sheet,
                    player_pos.translation,
                    0.05,
                    &explosion_audio,
                    &audio,
                );

                game_state.set(GameState::GameOver).unwrap();
                break;
            }
        }
    }
}

fn slide_out(
    mut q: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
    mut state: ResMut<State<GameState>>,
) {
    let mut player_pos = q.single_mut();
    player_pos.translation.x += SPEED * time.delta_seconds() / SLIDE_OUT_SLOWDOWN;

    if player_pos.translation.x > SIZE_X / 2.0 {
        state.set(GameState::Game).unwrap();
    }
}

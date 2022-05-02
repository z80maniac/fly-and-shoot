// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

use crate::anim::{AnimationTimer, AssetsLoading};
use crate::bullet::{Bullet, BulletRes, BulletTimer, HitBox};
use crate::collision::{DestroyOutsideScreen, Screen};
use crate::explosion::{Explosion, ExplosionAudio, ExplosionSheet};
use crate::player::{Player, PlayerBullet};
use crate::score::Score;
use crate::{AnimPlugin, GameState};
use bevy::prelude::*;
use bevy::sprite::collide_aabb::collide;
use rand::Rng;
use std::time::Duration;

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_startup_system(setup)
            .add_system(movement)
            .add_system(bullet_hit)
            .add_system_set(
                SystemSet::on_update(GameState::Game)
                    .with_system(process_spawn)
                    .with_system(attack),
            );
    }
}

#[derive(Component)]
pub struct Enemy {
    speed: Vec3,
}

#[derive(Component)]
pub struct EnemySpawn {
    timer: Timer,
}

#[derive(Component)]
pub struct EnemyBullet;

#[derive(Component)]
pub struct EnemiesContainer;

pub struct EnemyGraphics {
    ship_atlas: Handle<TextureAtlas>,
    exhaust_atlas: Handle<TextureAtlas>,
}

const WIDTH: f32 = 150.0;
const HEIGHT: f32 = 150.0;
const SIZE_X: f32 = 0.15;
const SIZE_Y: f32 = SIZE_X * HEIGHT / WIDTH;
const MIN_DISTANCE_TO_SHOOT: f32 = 0.5;
const BULLET_SPEED: f32 = 1.0;

const EXHAUST_WIDTH: f32 = 75.0;
const EXHAUST_HEIGHT: f32 = 64.0;
const EXHAUST_SIZE_X: f32 = 0.05;
const EXHAUST_SIZE_Y: f32 = EXHAUST_SIZE_X * EXHAUST_HEIGHT / EXHAUST_WIDTH;

fn setup(mut commands: Commands) {
    commands
        .spawn()
        .insert(EnemySpawn {
            timer: Timer::from_seconds(0.5, true),
        })
        .insert(Transform::default())
        .insert(GlobalTransform::default())
        .insert(Name::new("EnemySpawn"));

    commands
        .spawn()
        .insert(EnemiesContainer)
        .insert(Name::new("EnemiesContainer"));
}

fn process_spawn(
    commands: Commands,
    mut q: Query<&mut EnemySpawn>,
    score: Res<Score>,
    player_q: Query<&Transform, With<Player>>,
    time: Res<Time>,
    graphics: Res<EnemyGraphics>,
    win: Res<WindowDescriptor>,
) {
    let mut spawn_el = q.single_mut();
    spawn_el.timer.tick(time.delta());

    if spawn_el.timer.just_finished() {
        let mut rng = rand::thread_rng();
        let y = rng.gen_range(0.1..1.0);
        if let Ok(player) = player_q.get_single() {
            let player_pos = player.translation;

            spawn(commands, graphics, win, player_pos, &score, y);

            let timer_secs = score.interp(0.6, 0.3, 200);
            spawn_el
                .timer
                .set_duration(Duration::from_secs_f32(timer_secs));
        }
    }
}

fn spawn(
    mut commands: Commands,
    graphics: Res<EnemyGraphics>,
    win: Res<WindowDescriptor>,
    player_pos: Vec3,
    score: &Score,
    y: f32,
) {
    let mut enemy_sprite = TextureAtlasSprite::new(0);
    let mut rng = rand::thread_rng();

    let bounds = win.bounds_box_outside(Vec2::new(SIZE_X, SIZE_Y));

    let pos = Vec3::new(bounds.right, y, 100.0);
    let speed = player_pos - pos;
    let speed = Vec3::new(speed.x, speed.y, 0.0).normalize();
    let speed = speed * score.interp(0.5, 2.0, 200);

    let angle = rng.gen_range(-0.3..0.3);
    let rot = Quat::from_rotation_z(angle);
    let speed = rot.mul_vec3(speed);

    let bullet_period = score.interp(3.0, 1.5, 200);
    let bullet_delay = rng.gen_range(0.0..bullet_period);

    enemy_sprite.custom_size = Some(Vec2::new(SIZE_X, SIZE_Y));
    enemy_sprite.flip_x = true;
    let enemy = commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: enemy_sprite,
            texture_atlas: graphics.ship_atlas.clone(),
            transform: Transform {
                translation: pos,
                ..default()
            },
            ..default()
        })
        .insert(Enemy { speed })
        .insert(HitBox(Vec2::new(SIZE_X * 0.5, SIZE_Y * 0.5)))
        .insert(DestroyOutsideScreen {
            size: Vec2::new(SIZE_X, SIZE_Y),
        })
        .insert(BulletTimer::new_delayed(bullet_period, bullet_delay))
        .insert(Name::new("Enemy"))
        .insert(GlobalTransform::default())
        .id();

    let mut exhaust_sprite = TextureAtlasSprite::new(0);
    exhaust_sprite.custom_size = Some(Vec2::new(EXHAUST_SIZE_X, EXHAUST_SIZE_Y));
    let exhaust = commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: exhaust_sprite,
            texture_atlas: graphics.exhaust_atlas.clone(),
            transform: Transform {
                translation: Vec3::new(0.085, -0.01, -1.0),
                ..default()
            },
            ..default()
        })
        .insert(AnimationTimer::infinite(0.1))
        .insert(Name::new("EnemyExhaust"))
        .id();

    commands.entity(enemy).add_child(exhaust);
}

fn load_assets(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut loading: ResMut<AssetsLoading>,
) {
    let ship_atlas = AnimPlugin::load_atlas(
        "enemy.png",
        Vec2::new(WIDTH, HEIGHT),
        1,
        1,
        &assets,
        &mut atlases,
        &mut loading,
    );

    let exhaust_atlas = AnimPlugin::load_atlas(
        "enemy_exhaust.png",
        Vec2::new(EXHAUST_WIDTH, EXHAUST_HEIGHT),
        2,
        2,
        &assets,
        &mut atlases,
        &mut loading,
    );

    commands.insert_resource(EnemyGraphics {
        ship_atlas,
        exhaust_atlas,
    });
}

fn bullet_hit(
    mut commands: Commands,
    enemy_query: Query<(Entity, &Transform, &HitBox), With<Enemy>>,
    player_bullet_query: Query<(Entity, &Transform, &HitBox), With<PlayerBullet>>,
    mut score: ResMut<Score>,
    explosion_sheet: Res<ExplosionSheet>,
    explosion_audio: Res<ExplosionAudio>,
    audio: Res<Audio>,
) {
    for (enemy, enemy_pos, enemy_hitbox) in enemy_query.iter() {
        for (bullet, bullet_pos, bullet_hitbox) in player_bullet_query.iter() {
            if collide(
                enemy_pos.translation,
                enemy_hitbox.0,
                bullet_pos.translation,
                bullet_hitbox.0,
            )
            .is_some()
            {
                commands.entity(enemy).despawn_recursive();
                commands.entity(bullet).despawn_recursive();

                Explosion::spawn(
                    &mut commands,
                    &explosion_sheet,
                    enemy_pos.translation,
                    0.05,
                    &explosion_audio,
                    &audio,
                );

                score.inc();
                break;
            }
        }
    }
}

fn movement(mut q: Query<(&Enemy, &mut Transform)>, time: Res<Time>) {
    for (enemy, mut transform) in q.iter_mut() {
        transform.translation += enemy.speed * time.delta_seconds();
    }
}

fn attack(
    mut commands: Commands,
    mut q: Query<(&Transform, &mut BulletTimer), With<Enemy>>,
    player_q: Query<&Transform, With<Player>>,
    time: Res<Time>,
    bullet_res: Res<BulletRes>,
    audio: Res<Audio>,
) {
    if let Ok(player_transform) = player_q.get_single() {
        let player_pos = Vec3::new(
            player_transform.translation.x,
            player_transform.translation.y,
            0.3,
        );

        let mut rng = rand::thread_rng();

        for (enemy_transform, mut bullet_timer) in q.iter_mut() {
            bullet_timer.process(time.delta());

            if bullet_timer.can_shoot && enemy_transform.translation.x > player_pos.x {
                let enemy_pos = Vec3::new(
                    enemy_transform.translation.x,
                    enemy_transform.translation.y,
                    0.3,
                );
                let dist = enemy_pos.distance(player_pos);

                let max_diff_y = 0.25;
                let diff_y = rng.gen_range(-max_diff_y..max_diff_y);
                let mut target_pos = player_pos.clone();
                target_pos.y += diff_y;

                if dist > MIN_DISTANCE_TO_SHOOT {
                    let bullet = Bullet::spawn(
                        &mut commands,
                        &bullet_res.enemy,
                        enemy_pos,
                        target_pos,
                        BULLET_SPEED,
                        Color::WHITE,
                        &audio,
                    );
                    commands
                        .entity(bullet)
                        .insert(EnemyBullet)
                        .insert(Name::new("EnemyBullet"));
                    bullet_timer.shoot();
                }
            }
        }
    }
}

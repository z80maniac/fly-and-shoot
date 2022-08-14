// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

use std::time::Duration;

use crate::anim::AssetsLoading;
use crate::collision::DestroyOutsideScreen;
use crate::AnimPlugin;
use bevy::prelude::*;

pub struct BulletPlugin;

impl Plugin for BulletPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system(movement);
    }
}

pub struct BulletResInfo {
    atlas: Handle<TextureAtlas>,
    audio: Handle<AudioSource>,
    sprite_size: Vec2,
    collision_size: Vec2,
    audio_volume: f32,
}

impl BulletResInfo {
    fn load(
        assets: &Res<AssetServer>,
        atlases: &mut ResMut<Assets<TextureAtlas>>,
        loading: &mut ResMut<AssetsLoading>,
        image_filename: &str,
        audio_filename: &str,
        atlas_size: Vec2,
        sprite_size: Vec2,
        collision_size: Vec2,
        audio_volume: f32,
    ) -> Self {
        let atlas = AnimPlugin::load_atlas(
            image_filename,
            Vec2::new(atlas_size.x, atlas_size.y),
            1,
            1,
            assets,
            atlases,
            loading,
        );
        let audio = assets.load(audio_filename);
        loading.push(audio.clone_untyped());
        return Self {
            atlas,
            audio,
            sprite_size,
            collision_size,
            audio_volume,
        };
    }
}

pub struct BulletRes {
    pub player: BulletResInfo,
    pub enemy: BulletResInfo,
}

#[derive(Component)]
pub struct Bullet {
    speed: Vec3,
}

impl Bullet {
    pub fn spawn(
        commands: &mut Commands,
        res_info: &BulletResInfo,
        starting_point: Vec3,
        target_point: Vec3,
        speed: f32,
        color: Color,
        audio: &Res<Audio>,
    ) -> Entity {
        let mut sprite = TextureAtlasSprite::new(0);

        sprite.custom_size = Some(res_info.sprite_size);
        sprite.color = color;
        let speed_vec = (target_point - starting_point).normalize() * speed;

        let sprite_bundle = SpriteSheetBundle {
            sprite,
            texture_atlas: res_info.atlas.clone(),
            transform: Transform {
                translation: starting_point,
                rotation: Quat::from_rotation_arc(
                    Vec3::X,
                    (target_point - starting_point).normalize(),
                ),
                ..default()
            },
            ..default()
        };

        let entity = commands
            .spawn_bundle(sprite_bundle)
            .insert(Bullet { speed: speed_vec })
            .insert(HitBox(res_info.collision_size))
            .insert(DestroyOutsideScreen {
                size: res_info.sprite_size,
            })
            .id();

        audio.play_with_settings(
            res_info.audio.clone(),
            PlaybackSettings {
                volume: res_info.audio_volume,
                ..default()
            },
        );
        return entity;
    }
}

#[derive(Component)]
pub struct BulletTimer {
    timer: Timer,
    pub can_shoot: bool,
}

impl BulletTimer {
    pub fn new(secs: f32) -> Self {
        return BulletTimer {
            timer: Timer::from_seconds(secs, true),
            can_shoot: true,
        };
    }

    pub fn new_delayed(period_secs: f32, delay_secs: f32) -> Self {
        let mut timer = BulletTimer {
            timer: Timer::from_seconds(period_secs, true),
            can_shoot: false,
        };
        let millis = ((period_secs - delay_secs) * 1000.0) as u64;
        timer.process(Duration::from_millis(millis));
        return timer;
    }

    pub fn shoot(&mut self) {
        self.can_shoot = false;
    }

    pub fn process(&mut self, delta: Duration) {
        if self.can_shoot {
            return;
        }

        self.timer.tick(delta);
        if self.timer.just_finished() {
            self.can_shoot = true;
        }
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct HitBox(pub Vec2);

fn load_assets(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut loading: ResMut<AssetsLoading>,
) {
    let player_atlas_size = Vec2::new(141.0, 129.0);
    let player_sprite_size = player_atlas_size / 141.0 * 0.15;
    let player = BulletResInfo::load(
        &assets,
        &mut atlases,
        &mut loading,
        "player_bullet.png",
        "player_bullet.ogg",
        player_atlas_size,
        player_sprite_size,
        player_sprite_size * 0.1,
        0.25,
    );

    let enemy_atlas_size = Vec2::new(325.0, 238.0);
    let enemy_sprite_size = player_atlas_size / 325.0 * 0.25;
    let enemy = BulletResInfo::load(
        &assets,
        &mut atlases,
        &mut loading,
        "enemy_bullet.png",
        "enemy_bullet.ogg",
        enemy_atlas_size,
        enemy_sprite_size,
        enemy_sprite_size * 0.1,
        0.75,
    );

    commands.insert_resource(BulletRes { player, enemy });
}

fn movement(mut q: Query<(&Bullet, &mut Transform)>, time: Res<Time>) {
    for (bullet, mut transform) in &mut q {
        transform.translation += bullet.speed * time.delta_seconds();
    }
}

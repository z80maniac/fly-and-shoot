// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

use std::f32::consts::PI;

use crate::anim::{AnimationTimer, AssetsLoading};
use crate::AnimPlugin;
use bevy::prelude::*;
use rand::Rng;

pub struct ExplosionPlugin;

impl Plugin for ExplosionPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
            .add_system(process);
    }
}

#[derive(Component)]
pub struct Explosion;

impl Explosion {
    pub fn spawn(
        commands: &mut Commands,
        sheet: &Res<ExplosionSheet>,
        pos: Vec3,
        frame_duration: f32,
        explosion_audio: &Res<ExplosionAudio>,
        audio: &Res<Audio>,
    ) {
        let mut sprite = TextureAtlasSprite::new(0);

        let angle = rand::thread_rng().gen_range(0.0..PI);

        sprite.custom_size = Some(Vec2::new(SIZE_X, SIZE_Y));
        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite,
                texture_atlas: sheet.0.clone(),
                transform: Transform {
                    translation: pos,
                    rotation: Quat::from_rotation_z(angle),
                    ..default()
                },
                ..default()
            })
            .insert(Explosion)
            .insert(AnimationTimer::finite(
                frame_duration,
                SHEET_COLUMNS * SHEET_ROWS,
            ))
            .insert(Name::new("Explosion"));

        audio.play_with_settings(
            explosion_audio.0.clone(),
            PlaybackSettings {
                volume: 0.25,
                ..default()
            },
        );
    }
}

pub struct ExplosionSheet(Handle<TextureAtlas>);

pub struct ExplosionAudio(Handle<AudioSource>);

const WIDTH: f32 = 128.0;
const HEIGHT: f32 = 128.0;
const SIZE_X: f32 = 0.5;
const SIZE_Y: f32 = 0.5;
const SHEET_COLUMNS: usize = 4;
const SHEET_ROWS: usize = 4;

fn load_assets(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut loading: ResMut<AssetsLoading>,
) {
    let handle = AnimPlugin::load_atlas(
        "explosion.png",
        Vec2::new(WIDTH, HEIGHT),
        SHEET_COLUMNS,
        SHEET_ROWS,
        &assets,
        &mut atlases,
        &mut loading,
    );
    commands.insert_resource(ExplosionSheet(handle));

    let audio = assets.load("explosion.ogg");
    loading.push(audio.clone_untyped());
    commands.insert_resource(ExplosionAudio(audio));
}

fn process(mut commands: Commands, q: Query<(Entity, &AnimationTimer), With<Explosion>>) {
    for (explosion, timer) in &q {
        if timer.timer.paused() {
            commands.entity(explosion).despawn_recursive();
        }
    }
}

// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

use crate::audio::AudioTrack;
use crate::{anim::AssetsLoading, state::GameState, AnimPlugin};
use bevy::{audio::AudioSink, prelude::*};

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, setup)
            .add_system_set(SystemSet::on_enter(GameState::TitleFlyIn).with_system(stop_bg_music))
            .add_system_set(
                SystemSet::on_enter(GameState::PlayerSlideOut).with_system(start_bg_music),
            )
            .add_system(movement);
    }
}

const WIDTH: f32 = 1024.0;
const HEIGHT: f32 = 512.0;
const SIZE_Y: f32 = 1.0;
const SIZE_X: f32 = SIZE_Y * WIDTH / HEIGHT;
const SPEED: f32 = 0.03;

pub struct BackgroundSheet(Handle<TextureAtlas>);

#[derive(Deref, DerefMut)]
pub struct BackgroundAudio(pub AudioTrack);

#[derive(Component)]
pub struct Background {
    pub initial_x: f32,
}

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
    mut loading: ResMut<AssetsLoading>,
) {
    let handle = AnimPlugin::load_atlas(
        "background.png",
        Vec2::new(WIDTH, HEIGHT),
        1,
        1,
        &assets,
        &mut atlases,
        &mut loading,
    );
    commands.insert_resource(BackgroundSheet(handle.clone()));

    let mut sprite = TextureAtlasSprite::new(0);
    sprite.custom_size = Some(Vec2::new(SIZE_X, SIZE_Y));

    for i in 1..=2 {
        let initial_x = SIZE_X / 2.0 + SIZE_X * (i - 1) as f32 - 0.001 * (i - 1) as f32;
        commands
            .spawn_bundle(SpriteSheetBundle {
                sprite: sprite.clone(),
                texture_atlas: handle.clone(),
                transform: Transform {
                    translation: Vec3::new(initial_x, SIZE_Y / 2.0, 0.0),
                    ..default()
                },
                ..default()
            })
            .insert(Background { initial_x })
            .insert(Name::new(format!("Background{}", i)));
    }

    let bg_audio = assets.load("background.ogg");
    loading.push(bg_audio.clone_untyped());
    commands.insert_resource(BackgroundAudio(AudioTrack::new(bg_audio)));
}

fn start_bg_music(
    mut bg_audio: ResMut<BackgroundAudio>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>,
) {
    bg_audio.play(&audio, &audio_sinks);
}

fn stop_bg_music(bg_audio: Res<BackgroundAudio>, audio_sinks: Res<Assets<AudioSink>>) {
    bg_audio.stop(&audio_sinks);
}

fn movement(mut q: Query<(&mut Transform, &Background)>, time: Res<Time>) {
    for (mut bg_pos, bg) in &mut q {
        let mut new_x = bg_pos.translation.x - SPEED * time.delta_seconds();
        let min_x = bg.initial_x - SIZE_X;

        if new_x < min_x {
            new_x = bg.initial_x - (min_x - new_x);
        }

        bg_pos.translation.x = new_x;
    }
}

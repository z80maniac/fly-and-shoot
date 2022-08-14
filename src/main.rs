// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

mod anim;
mod audio;
mod background;
mod bullet;
mod collision;
mod debug;
mod enemy;
mod explosion;
mod game_over;
mod player;
mod score;
mod state;
mod title;

use crate::anim::AnimPlugin;
use crate::background::BackgroundPlugin;
use crate::bullet::BulletPlugin;
use crate::collision::CollisionPlugin;
use crate::debug::DebugPlugin;
use crate::enemy::EnemyPlugin;
use crate::explosion::ExplosionPlugin;
use crate::game_over::GameOverPlugin;
use crate::player::PlayerPlugin;
use crate::score::ScorePlugin;
use crate::state::GameState;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use bevy::window::PresentMode;
use title::TitlePlugin;

const HEIGHT: f32 = 1080.0;
const WIDTH: f32 = 1920.0;
const RESOLUTION: f32 = WIDTH / HEIGHT;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(WindowDescriptor {
            width: WIDTH,
            height: HEIGHT,
            present_mode: PresentMode::Fifo, // VSYNC
            resizable: false,
            title: "Fly and Shoot".to_string(),
            ..default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .add_state(GameState::Loading)
        .add_plugin(AnimPlugin)
        .add_plugin(EnemyPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(BulletPlugin)
        .add_plugin(ExplosionPlugin)
        .add_plugin(CollisionPlugin)
        .add_plugin(BackgroundPlugin)
        .add_plugin(ScorePlugin)
        .add_plugin(TitlePlugin)
        .add_plugin(GameOverPlugin)
        .add_plugin(DebugPlugin)
        .add_startup_system(spawn_camera)
        .run();
}

fn spawn_camera(mut commands: Commands) {
    commands
        .spawn_bundle(Camera2dBundle {
            projection: OrthographicProjection {
                top: 1.0,
                bottom: 0.0,
                left: 0.0,
                right: RESOLUTION,
                scaling_mode: ScalingMode::None,
                ..default()
            },
            ..default()
        })
        .insert(Name::new("Camera"));
}

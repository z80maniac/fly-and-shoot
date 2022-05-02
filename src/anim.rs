// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

use bevy::{asset::LoadState, prelude::*};

use crate::state::GameState;

#[derive(Component)]
pub struct AnimationTimer {
    pub timer: Timer,
    pub frames_left: usize,
    pub is_infinite: bool,
}

impl AnimationTimer {
    pub fn infinite(period: f32) -> Self {
        return Self {
            timer: Timer::from_seconds(period, true),
            frames_left: 0,
            is_infinite: true,
        };
    }

    pub fn finite(period: f32, frames: usize) -> Self {
        return Self {
            timer: Timer::from_seconds(period, true),
            frames_left: frames,
            is_infinite: false,
        };
    }
}

pub struct AnimPlugin;

impl Plugin for AnimPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system_to_stage(StartupStage::PreStartup, setup)
            .add_system_set(
                SystemSet::on_update(GameState::Loading).with_system(check_assets_loaded),
            )
            .add_system(animate_sprite)
            .insert_resource(AssetsLoading::new());
    }
}

impl AnimPlugin {
    pub fn load_atlas(
        asset_filename: &str,
        tile_size: Vec2,
        columns: usize,
        rows: usize,
        assets: &Res<AssetServer>,
        atlases: &mut ResMut<Assets<TextureAtlas>>,
        loading: &mut ResMut<AssetsLoading>,
    ) -> Handle<TextureAtlas> {
        let image = assets.load(asset_filename);
        loading.0.push(image.clone_untyped());

        let atlas = TextureAtlas::from_grid(image, tile_size, columns, rows);
        let atlas_handle = atlases.add(atlas);
        return atlas_handle;
    }

    pub fn text_bundle(
        font: &Res<MainFont>,
        text: &str,
        font_size: f32,
        pos: Vec3,
    ) -> Text2dBundle {
        return Text2dBundle {
            text: Text::with_section(
                text,
                TextStyle {
                    font: font.0.clone(),
                    font_size,
                    color: Color::WHITE,
                },
                TextAlignment {
                    vertical: VerticalAlign::Center,
                    horizontal: HorizontalAlign::Center,
                },
            ),
            transform: Transform {
                translation: pos,
                scale: Vec3::splat(0.001),
                ..default()
            },
            visibility: Visibility { is_visible: false },
            ..default()
        };
    }
}

#[derive(Deref, DerefMut)]
pub struct AssetsLoading(pub Vec<HandleUntyped>);

impl AssetsLoading {
    fn new() -> Self {
        return Self(Vec::new());
    }

    fn is_loaded(&self, server: &AssetServer) -> bool {
        let state = server.get_group_load_state(self.iter().map(|h| h.id));
        return state == LoadState::Loaded;
    }
}

pub struct MainFont(Handle<Font>);

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    let font_asset = asset_server.load("font.ttf");
    loading.push(font_asset.clone_untyped());
    let res = MainFont(font_asset);
    commands.insert_resource(res);
}

fn check_assets_loaded(
    mut commands: Commands,
    loading: ResMut<AssetsLoading>,
    server: Res<AssetServer>,
    mut state: ResMut<State<GameState>>,
) {
    if loading.is_loaded(&server) {
        commands.remove_resource::<AssetsLoading>();
        state.set(GameState::TitleFlyIn).unwrap();
    }
}

fn animate_sprite(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &Handle<TextureAtlas>,
    )>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        if timer.timer.paused() {
            continue;
        }

        timer.timer.tick(time.delta());
        if timer.timer.just_finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            let mut next_index = sprite.index + 1;
            if next_index >= texture_atlas.textures.len() {
                if timer.is_infinite {
                    next_index = 0;
                } else {
                    next_index = sprite.index;
                    timer.timer.pause();
                }
            }
            sprite.index = next_index;
        }
    }
}

// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

use bevy::{asset::LoadState, prelude::*};

use crate::state::GameState;
use crate::collision::Screen;

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
        font: &Handle<Font>,
        text: &str,
        font_size: f32,
        pos: Vec3,
    ) -> Text2dBundle {
        return Text2dBundle {
            text: Text::with_section(
                text,
                TextStyle {
                    font: font.clone(),
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

    fn loaded_state(&self, server: &AssetServer) -> LoadState {
        let state = server.get_group_load_state(self.iter().map(|h| h.id));
        return state;
    }
}

pub struct MainFont(pub Handle<Font>);

#[derive(Component)]
pub struct LoadingText;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
    win: Res<WindowDescriptor>
) {
    let font_asset = asset_server.load("font.ttf");
    loading.push(font_asset.clone_untyped());

    let mut loading_text = AnimPlugin::text_bundle(
        &font_asset, "LOADING...", 32.0, win.middle_with_z(1.0));
    loading_text.visibility.is_visible = true;
    commands.spawn_bundle(loading_text).insert(LoadingText);

    let res = MainFont(font_asset);
    commands.insert_resource(res);
}

fn check_assets_loaded(
    mut commands: Commands,
    loading: ResMut<AssetsLoading>,
    server: Res<AssetServer>,
    mut state: ResMut<State<GameState>>,
    mut loading_q: Query<(Entity, &mut Text), With<LoadingText>>,
) {
    match loading.loaded_state(&server) {
        LoadState::Loaded => {
            commands.remove_resource::<AssetsLoading>();
            let (loading_text_entity, _) = loading_q.single_mut();
            commands.entity(loading_text_entity).despawn_recursive();
            state.set(GameState::TitleFlyIn).unwrap();
        },
        LoadState::Failed => {
            let (_, mut loading_text) = loading_q.single_mut();
            let mut section = loading_text.sections.first_mut().unwrap();
            section.value = "LOADING FAILED! CHECK THE CONSOLE.".to_string();
            section.style.color = Color::RED;
        },
        _ => {}
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

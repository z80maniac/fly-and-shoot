// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

use crate::audio::AudioTrack;
use crate::collision::Screen;
use crate::{
    anim::{AnimPlugin, AssetsLoading, MainFont},
    score::Score,
    state::GameState,
};
use bevy::{audio::AudioSink, prelude::*};
use std::{f32::consts::PI, time::Duration};

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system_set(
                SystemSet::on_enter(GameState::TitleFlyIn)
                    .with_system(show_title)
                    .with_system(show_instructions)
                    .with_system(start_audio),
            )
            .add_system_set(SystemSet::on_update(GameState::TitleFlyIn).with_system(fly_in))
            .add_system_set(
                SystemSet::on_update(GameState::TitleInstructionsFlyIn)
                    .with_system(instructions_fly_in)
                    .with_system(animate_title),
            )
            .add_system_set(SystemSet::on_update(GameState::TitleFlyOut).with_system(fly_out))
            .add_system_set(SystemSet::on_enter(GameState::Title).with_system(show_action_text))
            .add_system_set(
                SystemSet::on_update(GameState::Title)
                    .with_system(animate_title)
                    .with_system(animate_action_text)
                    .with_system(wait_for_enter),
            );
    }
}

const TITLE_SHADOWS: u8 = 3;
const SHADOW_RADIUS: f32 = 0.0035;
const SHADOW_ALPHA: f32 = 0.5;
const SHADOW_SPEED: f32 = 10.0;
const SHADOW_OUTER_RADIUS: f32 = 1.5;

#[derive(Component)]
pub struct TitleShadow {
    index: u8,
}

impl TitleShadow {
    fn angle_offset(&self) -> f32 {
        return PI * 2.0 / TITLE_SHADOWS as f32 * self.index as f32;
    }

    fn pos_for_time(&self, secs: f64, radius: f32, win: &WindowDescriptor) -> Vec3 {
        let center = Vec3::new(
            win.middle_x(),
            win.middle_y() + 0.25,
            0.1 + 0.01 * (self.index as f32),
        );
        let angle = secs as f32 * SHADOW_SPEED;
        let final_angle = angle + self.angle_offset();
        let offset = Vec3::new(final_angle.cos() * radius, final_angle.sin() * radius, 0.0);
        let final_pos = center + offset;
        return final_pos;
    }

    fn pos_for_flyin_time(&self, secs: f64, win: &WindowDescriptor, timer: &TitleTimer) -> Vec3 {
        let ratio = timer.elapsed_secs() / timer.duration().as_secs_f32();
        let radius = SHADOW_OUTER_RADIUS - (SHADOW_OUTER_RADIUS - SHADOW_RADIUS) * ratio;
        let pos = self.pos_for_time(secs, radius, win);
        return pos;
    }

    fn pos_for_flyout_time(&self, secs: f64, win: &WindowDescriptor, timer: &TitleTimer) -> Vec3 {
        let ratio = timer.elapsed_secs() / timer.duration().as_secs_f32();
        let radius = SHADOW_RADIUS + (SHADOW_OUTER_RADIUS - SHADOW_RADIUS) * ratio;
        let pos = self.pos_for_time(secs, radius, win);
        return pos;
    }

    fn color(&self) -> Color {
        let hue = (self.angle_offset()).sin() * 180.0 + 180.0;
        return Color::Hsla {
            hue,
            saturation: 1.0,
            lightness: 0.5,
            alpha: SHADOW_ALPHA,
        };
    }
}

#[derive(Component)]
pub struct TitleInstructions;

#[derive(Component)]
pub struct TitleActionText;

#[derive(Component, Deref, DerefMut)]
pub struct TitleTimer(pub Timer);

pub struct TitleActionAudio(Handle<AudioSource>);

#[derive(Deref, DerefMut)]
pub struct TitleAudio(pub AudioTrack);

fn setup(
    mut commands: Commands,
    font: Res<MainFont>,
    win: Res<WindowDescriptor>,
    assets: Res<AssetServer>,
    mut loading: ResMut<AssetsLoading>,
) {
    for i in 0..TITLE_SHADOWS {
        let title = AnimPlugin::text_bundle(&font.0, "FLY AND SHOOT", 180.0, Vec3::ZERO);
        commands
            .spawn_bundle(title)
            .insert(TitleShadow { index: i })
            .insert(Name::new(format!("TitleShadow{}", i)));
    }

    commands
        .spawn()
        .insert(TitleTimer(Timer::from_seconds(2.0, true)))
        .insert(Name::new("TitleTimer"));

    let instructions = AnimPlugin::text_bundle(
        &font.0,
        "WASD - MOVEMENT\nM - ATTACK",
        50.0,
        Vec3::new(win.middle_x(), win.middle_y() - 0.25, 0.1),
    );
    commands
        .spawn_bundle(instructions)
        .insert(TitleInstructions)
        .insert(Name::new("Instructions"));

    let action_text = AnimPlugin::text_bundle(
        &font.0,
        "PRESS ENTER",
        100.0,
        Vec3::new(win.middle_x(), win.middle_y() - 0.35, 0.1),
    );
    commands
        .spawn_bundle(action_text)
        .insert(TitleActionText)
        .insert(Name::new("ActionText"));

    let action_audio = assets.load("start.ogg");
    loading.push(action_audio.clone_untyped());
    commands.insert_resource(TitleActionAudio(action_audio));

    let bg_audio = assets.load("title.ogg");
    loading.push(bg_audio.clone_untyped());
    commands.insert_resource(TitleAudio(AudioTrack::new(bg_audio)));
}

fn show_title(
    mut shadows_q: Query<(&TitleShadow, &mut Transform, &mut Visibility, &mut Text)>,
    time: Res<Time>,
    win: Res<WindowDescriptor>,
) {
    for (shadow, mut shadow_pos, mut visibility, mut text) in &mut shadows_q {
        let new_pos = shadow.pos_for_time(time.seconds_since_startup(), SHADOW_RADIUS, &win);
        shadow_pos.translation = new_pos;
        visibility.is_visible = true;

        let mut section = text.sections.first_mut().unwrap();
        section.style.color = shadow.color();
    }
}

fn show_instructions(mut q: Query<(&mut Text, &mut Visibility), With<TitleInstructions>>) {
    let (mut text, mut visibility) = q.single_mut();
    visibility.is_visible = true;
    text.sections.first_mut().unwrap().style.color.set_a(0.0);
}

fn show_action_text(mut q: Query<&mut Visibility, With<TitleActionText>>) {
    let mut visibility = q.single_mut();
    visibility.is_visible = true;
}

fn start_audio(
    mut bg_audio: ResMut<TitleAudio>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>,
) {
    bg_audio.play(&audio, &audio_sinks);
}

fn fly_in(
    mut shadows_q: Query<(&TitleShadow, &mut Transform, &mut Text)>,
    mut timer_q: Query<&mut TitleTimer>,
    time: Res<Time>,
    win: Res<WindowDescriptor>,
    mut state: ResMut<State<GameState>>,
) {
    let mut timer = timer_q.single_mut();
    timer.tick(time.delta());
    if timer.just_finished() {
        timer.set_duration(Duration::from_secs_f32(0.5));
        state.set(GameState::TitleInstructionsFlyIn).unwrap();
        return;
    }

    let ratio = timer.elapsed_secs() / timer.duration().as_secs_f32();

    for (shadow, mut shadow_pos, mut text) in &mut shadows_q {
        let new_pos = shadow.pos_for_flyin_time(time.seconds_since_startup(), &win, &timer);
        shadow_pos.translation = new_pos;

        let section = text.sections.first_mut().unwrap();
        section.style.color.set_a(ratio * SHADOW_ALPHA);
    }
}

fn instructions_fly_in(
    mut instructions_q: Query<(&mut Text, &mut Transform), With<TitleInstructions>>,
    mut timer_q: Query<&mut TitleTimer>,
    time: Res<Time>,
    mut state: ResMut<State<GameState>>,
) {
    let mut timer = timer_q.single_mut();
    timer.tick(time.delta());
    if timer.just_finished() {
        timer.set_duration(Duration::from_secs_f32(2.0));
        state.set(GameState::Title).unwrap();
        return;
    }

    let ratio = timer.elapsed_secs() / timer.duration().as_secs_f32();
    let (mut text, mut transform) = instructions_q.single_mut();
    text.sections.first_mut().unwrap().style.color.set_a(ratio);
    let y_offset = ratio / 10.0;
    transform.translation.y = 0.35 + y_offset;
}

fn fly_out(
    mut timer_q: Query<&mut TitleTimer>,
    mut text_set: ParamSet<(
        Query<(&TitleShadow, &mut Transform, &mut Text, &mut Visibility)>,
        Query<(&mut Text, &mut Transform, &mut Visibility), With<TitleInstructions>>,
        Query<&mut Visibility, With<TitleActionText>>,
    )>,
    time: Res<Time>,
    win: Res<WindowDescriptor>,
    mut state: ResMut<State<GameState>>,
) {
    let mut timer = timer_q.single_mut();
    timer.tick(time.delta());

    if timer.just_finished() {
        state.set(GameState::PlayerSlideOut).unwrap();

        for (_, _, _, mut shadow_visibility) in &mut text_set.p0() {
            shadow_visibility.is_visible = false;
        }

        let mut p1 = text_set.p1();
        let (_, _, mut instructions_visibility) = p1.single_mut();
        instructions_visibility.is_visible = false;

        text_set.p2().single_mut().is_visible = false;
        return;
    }

    let ratio = 1.0 - timer.elapsed_secs() / timer.duration().as_secs_f32();

    for (shadow, mut shadow_pos, mut text, _) in &mut text_set.p0() {
        let new_pos = shadow.pos_for_flyout_time(time.seconds_since_startup(), &win, &timer);
        shadow_pos.translation = new_pos;

        let section = text.sections.first_mut().unwrap();
        section.style.color.set_a(ratio * SHADOW_ALPHA);
    }

    let mut p1 = text_set.p1();
    let (mut instructions_text, mut instructions_pos, _) = p1.single_mut();
    instructions_text
        .sections
        .first_mut()
        .unwrap()
        .style
        .color
        .set_a(ratio);
    let y_offset = ratio / 10.0;
    instructions_pos.translation.y = 0.35 + y_offset;

    text_set.p2().single_mut().is_visible = (ratio * 50.0) as i32 % 2 == 1;
}

fn animate_title(
    mut shadows_q: Query<(&TitleShadow, &mut Transform)>,
    time: Res<Time>,
    win: Res<WindowDescriptor>,
) {
    for (shadow, mut shadow_pos) in &mut shadows_q {
        let new_pos = shadow.pos_for_time(time.seconds_since_startup(), SHADOW_RADIUS, &win);
        shadow_pos.translation = new_pos;
    }
}

fn animate_action_text(mut q: Query<&mut Text, With<TitleActionText>>, time: Res<Time>) {
    let r = time.seconds_since_startup().sin().abs();
    q.single_mut()
        .sections
        .first_mut()
        .unwrap()
        .style
        .color
        .set_r(r as f32)
        .set_g(1.0 - r as f32);
}

fn wait_for_enter(
    mut kbd: ResMut<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
    action_audio: Res<TitleActionAudio>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>,
    bg_audio: Res<TitleAudio>,
    mut score: ResMut<Score>,
) {
    if kbd.just_pressed(KeyCode::Return) {
        bg_audio.stop(&audio_sinks);

        audio.play_with_settings(action_audio.0.clone(), PlaybackSettings { ..default() });

        score.clear();
        state.set(GameState::TitleFlyOut).unwrap();
        kbd.clear();
    }
}

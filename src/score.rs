// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

use bevy::prelude::*;

use crate::{
    anim::{AnimPlugin, MainFont},
    state::GameState,
};

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system_set(SystemSet::on_enter(GameState::TitleFlyIn).with_system(hide_score_text))
            .add_system_set(
                SystemSet::on_enter(GameState::Game).with_system(setup_score_text_for_game),
            )
            .add_system_set(
                SystemSet::on_enter(GameState::GameOver)
                    .with_system(setup_score_text_for_game_over),
            )
            .add_system_set(
                SystemSet::on_update(GameState::GameOver).with_system(blink_text_for_game_over),
            )
            .add_system_set(
                SystemSet::on_exit(GameState::GameOver).with_system(setup_score_text_for_game),
            )
            .add_system(update_score_text);
    }
}

#[derive(Component)]
pub struct ScoreText;

pub struct Score {
    pub score: u32,
}

impl Score {
    const ZERO: Self = Self { score: 0 };
    pub const CONTINUE_COST: u32 = 50;

    pub fn inc(&mut self) {
        self.score += 1;
    }

    pub fn clear(&mut self) {
        self.score = 0;
    }

    pub fn can_continue(&self) -> bool {
        return self.score >= Self::CONTINUE_COST;
    }

    pub fn buy_continue(&mut self) {
        self.score = self
            .score
            .checked_sub(Self::CONTINUE_COST)
            .or(Some(0))
            .unwrap();
    }

    pub fn interp(&self, start_val: f32, end_val: f32, max_score: u32) -> f32 {
        let diff = end_val - start_val;
        let raw_result = start_val + (self.score as f32) / (max_score as f32) * diff;
        let clamped_result = if start_val < end_val {
            raw_result.clamp(start_val, end_val)
        } else {
            raw_result.clamp(end_val, start_val)
        };
        return clamped_result;
    }
}

fn setup(mut commands: Commands, font: Res<MainFont>) {
    commands.insert_resource(Score::ZERO);

    let text = AnimPlugin::text_bundle(&font, "SCORE: 0123456789", 25.0, Vec3::new(0.0, 0.0, 0.1));

    commands
        .spawn_bundle(text)
        .insert(ScoreText)
        .insert(Name::new("Score"));
}

fn hide_score_text(mut q: Query<&mut Visibility, With<ScoreText>>) {
    q.single_mut().is_visible = false;
}

fn setup_score_text_for_game(
    mut q: Query<(&mut Transform, &mut Visibility, &mut Text), With<ScoreText>>,
) {
    let (mut transform, mut visible, mut text) = q.single_mut();

    visible.is_visible = true;
    text.alignment.horizontal = HorizontalAlign::Left;
    text.alignment.vertical = VerticalAlign::Top;
    transform.translation.x = 0.01;
    transform.translation.y = 1.0;

    let mut section = text.sections.first_mut().unwrap();
    section.style.font_size = 25.0;
    section.style.color = Color::WHITE;
}

fn update_score_text(mut q: Query<&mut Text, With<ScoreText>>, score: Res<Score>) {
    if score.is_changed() {
        for mut text in q.iter_mut() {
            let mut section = text.sections.first_mut().unwrap();
            section.value = format!("SCORE: {}", score.score);
        }
    }
}

fn setup_score_text_for_game_over(
    mut q: Query<(&mut Transform, &mut Text), With<ScoreText>>,
    win: Res<WindowDescriptor>,
) {
    let (mut transform, mut text) = q.single_mut();

    text.alignment.horizontal = HorizontalAlign::Center;
    text.alignment.vertical = VerticalAlign::Center;
    transform.translation.x = win.width / win.height / 2.0;
    transform.translation.y = 0.7;

    let mut section = text.sections.first_mut().unwrap();
    section.style.font_size = 120.0;
}

fn blink_text_for_game_over(mut q: Query<&mut Text, With<ScoreText>>, time: Res<Time>) {
    let mut text = q.single_mut();
    let mut section = text.sections.first_mut().unwrap();
    let t = time.seconds_since_startup();
    let color_value = t.sin().abs();
    section.style.color = Color::rgb(1.0, color_value as f32, color_value as f32);
}

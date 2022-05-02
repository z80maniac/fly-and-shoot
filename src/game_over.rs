// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

use crate::anim::MainFont;
use crate::bullet::Bullet;
use crate::collision::Screen;
use crate::enemy::Enemy;
use crate::explosion::Explosion;
use crate::player::Player;
use crate::score::{Score, ScoreText};
use crate::{AnimPlugin, GameState};
use bevy::prelude::*;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup)
            .add_system_set(
                SystemSet::on_enter(GameState::GameOver).with_system(show_game_over_text),
            )
            .add_system_set(
                SystemSet::on_update(GameState::GameOver)
                    .with_system(wait_for_continue)
                    .with_system(wait_for_exit)
                    .label("wait_for_key")
                    .after("start_new_game_timer_if_field_is_empty"),
            )
            .add_system_set(
                SystemSet::on_update(GameState::GameOverWaitingForEmptyField)
                    .with_system(start_new_game_timer_if_field_is_empty)
                    .label("start_new_game_timer_if_field_is_empty")
                    .after("start_new_game_on_timer"),
            )
            .add_system_set(
                SystemSet::on_update(GameState::GameOverWaitingForTimer)
                    .with_system(start_new_game_on_timer)
                    .label("start_new_game_on_timer"),
            );
    }
}

#[derive(Component)]
pub struct NewGameTimer {
    timer: Timer,
}

impl NewGameTimer {
    const DURATION: f32 = 0.5;

    fn new() -> Self {
        return Self {
            timer: Timer::from_seconds(Self::DURATION, true),
        };
    }
}

const GG_TEXT_SIZE: f32 = 60.0;
const GG_TEXT_Z: f32 = 1.0;

#[derive(Component)]
pub struct GameOverText;

impl GameOverText {
    fn gg_text(can_continue: bool) -> String {
        if can_continue {
            return format!(
                "-= GAME OVER =-\n\n\
                PRESS \"ENTER\" TO SPEND {} POINTS AND CONTINUE\n\n\
                PRESS \"Q\" TO EXIT",
                Score::CONTINUE_COST
            );
        }
        return "-= GAME OVER =-\n\n\
            PRESS \"ENTER\" FOR QUICK RESTART\n\n\
            PRESS \"Q\" TO EXIT"
            .to_string();
    }
}

fn setup(mut commands: Commands, font: Res<MainFont>, win: Res<WindowDescriptor>) {
    commands
        .spawn()
        .insert(NewGameTimer::new())
        .insert(Name::new("NewGameTimer"));

    let mut gg_text = AnimPlugin::text_bundle(
        &font,
        &GameOverText::gg_text(true),
        GG_TEXT_SIZE,
        win.middle_with_z(GG_TEXT_Z),
    );
    gg_text.text.alignment.vertical = VerticalAlign::Top;
    commands
        .spawn_bundle(gg_text)
        .insert(GameOverText)
        .insert(Name::new("GameOverText"));
}

fn show_game_over_text(
    mut q: Query<(&mut Visibility, &mut Text), With<GameOverText>>,
    score: Res<Score>,
) {
    let (mut visibility, mut text) = q.single_mut();
    text.sections.first_mut().unwrap().value = GameOverText::gg_text(score.can_continue());
    visibility.is_visible = true;
}

fn wait_for_continue(
    mut kbd: ResMut<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
    mut q: Query<&mut Visibility, With<GameOverText>>,
) {
    if kbd.just_pressed(KeyCode::Return) {
        state.set(GameState::GameOverWaitingForEmptyField).unwrap();
        q.single_mut().is_visible = false;
        kbd.clear();
    }
}

fn wait_for_exit(
    mut commands: Commands,
    mut kbd: ResMut<Input<KeyCode>>,
    mut state: ResMut<State<GameState>>,
    mut q: Query<&mut Visibility, With<GameOverText>>,
    mut score_q: Query<&mut Visibility, (With<ScoreText>, Without<GameOverText>)>,
    players: Query<Entity, With<Player>>,
    enemies: Query<Entity, With<Enemy>>,
    bullets: Query<Entity, With<Bullet>>,
    explosions: Query<Entity, With<Explosion>>,
) {
    if kbd.just_pressed(KeyCode::Q) {
        for e in players.iter() {
            commands.entity(e).despawn_recursive();
        }
        for e in enemies.iter() {
            commands.entity(e).despawn_recursive();
        }
        for e in bullets.iter() {
            commands.entity(e).despawn_recursive();
        }
        for e in explosions.iter() {
            commands.entity(e).despawn_recursive();
        }

        state.set(GameState::TitleFlyIn).unwrap();
        q.single_mut().is_visible = false;
        score_q.single_mut().is_visible = false;
        kbd.clear();
    }
}

fn start_new_game_timer_if_field_is_empty(
    players: Query<(), With<Player>>,
    enemies: Query<(), With<Enemy>>,
    bullets: Query<(), With<Bullet>>,
    explosions: Query<(), With<Explosion>>,
    mut state: ResMut<State<GameState>>,
) {
    if players.is_empty() && enemies.is_empty() && bullets.is_empty() && explosions.is_empty() {
        state.set(GameState::GameOverWaitingForTimer).unwrap();
    }
}

fn start_new_game_on_timer(
    mut state: ResMut<State<GameState>>,
    mut timer_query: Query<&mut NewGameTimer>,
    time: Res<Time>,
    mut score: ResMut<Score>,
) {
    let mut timer = timer_query.single_mut();
    timer.timer.tick(time.delta());
    if timer.timer.just_finished() {
        score.buy_continue();
        state.set(GameState::PlayerSlideOut).unwrap();
    }
}

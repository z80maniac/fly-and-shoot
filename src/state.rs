// SPDX-License-Identifier: GPL-3.0-only
// 🄯 2022, Alexey Parfenov <zxed@alkatrazstudio.net>

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum GameState {
    Loading,

    TitleFlyIn,
    TitleInstructionsFlyIn,
    Title,
    TitleFlyOut,

    PlayerSlideOut,
    Game,

    GameOver,
    GameOverWaitingForEmptyField,
    GameOverWaitingForTimer,
}

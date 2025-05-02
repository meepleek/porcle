//! The title screen that appears when the game starts.

use bevy::prelude::*;

use super::Screen;
use crate::{game::score::Score, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::GameOver), enter_game_over);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum BtnAction {
    Play,
}

fn enter_game_over(mut commands: Commands, score: Res<Score>) {
    commands.spawn((
        StateScoped(Screen::GameOver),
        widget::ui_root("game_over"),
        children![
            widget::label("GAME OVER"),
            widget::label(format!("SCORE: {}", score.0)),
            widget::button("TRY AGAIN", super::enter_screen_click_trigger(Screen::Game))
        ],
    ));
}

//! A credits screen that can be accessed from the title screen.

use bevy::prelude::*;

use super::Screen;
use crate::theme::widget;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Credits), enter_credits);
}

fn enter_credits(mut commands: Commands) {
    commands.spawn((
        StateScoped(Screen::Credits),
        widget::ui_root("credits"),
        children![
            widget::header("CREDITS"),
            widget::label("Didn't have time to put in game, sorry.\n Check the game's github page. I'll put it there a couple days after release."),
            widget::button("BACK", super::enter_screen_on_pointer_click(Screen::Title)),
        ]));
}

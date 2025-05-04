//! The title screen that appears when the game starts.

use bevy::prelude::*;

use super::Screen;
use crate::theme::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Title), enter_title)
        .add_systems(OnEnter(Screen::Exit), exit_app);
}

fn enter_title(mut commands: Commands) {
    commands.spawn((
        StateScoped(Screen::Title),
        widget::ui_root("menu"),
        children![
            widget::header("PORCLE"),
            widget::button("PLAY", super::enter_screen_on_pointer_click(Screen::Game)),
            widget::button(
                "TUTORIAL",
                super::enter_screen_on_pointer_click(Screen::Tutorial)
            ),
            widget::button(
                "CREDITS",
                super::enter_screen_on_pointer_click(Screen::Credits)
            ),
            #[cfg(not(target_family = "wasm"))]
            widget::button("EXIT", super::enter_screen_on_pointer_click(Screen::Exit)),
        ],
    ));
}

fn exit_app(mut app_exit: EventWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}

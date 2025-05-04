//! A credits screen that can be accessed from the title screen.

use bevy::prelude::*;

use super::Screen;
use crate::theme::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Tutorial), enter);
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum TutorialAction {
    Play,
}

fn enter(mut commands: Commands) {
    commands.spawn((
        StateScoped(Screen::Tutorial),
        widget::ui_root("tutorial"),
        // fixme: this has multiple Children components and causes a panic
        children![
            widget::header("HOW TO PLAY"),
            widget::label("Use your good ol' trusty Porcle to protect the core as long as you can!"),
            widget::label("Enemies will swarm you from all angles, so be sure make sure both of your gun and your saw blade & keep it going."),
            widget::header("MOUSE CONTROLS"),
            (
                widget::label("Move: aim ship"),
                widget::label("LMB: Shoot"),
                widget::label("RMB: Capture & release sawblade"),
                widget::label("R: Restart"),
                widget::label("ESC: Back to menu"),
                widget::header("GAMEPAD CONTROLS"),
                widget::label("LEFT or RIGHT STICK: aim ship"),
                widget::label("RIGHT BUMPER or CROSS/A: Shoot"),
                widget::label("LEFT BUMPER or SQUARE/X: Capture & release sawblade"),
            ),
            (
                widget::header("SHIP CYCLE CONTROLS"),
                widget::label("Cycle 2x clockwise: recall sawblade"),
                widget::label("Cycle 1x counter-clockwise: gain ammo based on current sawblade speed"),
            ),

            widget::button("PLAY", super::enter_screen_on_pointer_click(Screen::Game)),
        ]
    ));
}

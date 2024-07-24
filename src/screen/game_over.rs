//! The title screen that appears when the game starts.

use bevy::prelude::*;

use super::Screen;
use crate::ui::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::GameOver), enter_game_over)
        .add_systems(
            Update,
            handle_title_action.run_if(in_state(Screen::GameOver)),
        );
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum BtnAction {
    Play,
}

fn enter_game_over(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::GameOver))
        .with_children(|children| {
            children.label("GAME OVER");
            children.label("todo: score");
            children.button("Try again").insert(BtnAction::Play);
        });
}

fn handle_title_action(
    mut next_screen: ResMut<NextState<Screen>>,
    mut button_query: InteractionQuery<&BtnAction>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                BtnAction::Play => next_screen.set(Screen::Game),
            }
        }
    }
}

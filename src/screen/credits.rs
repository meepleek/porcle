//! A credits screen that can be accessed from the title screen.

use bevy::prelude::*;

use super::{NextTransitionedState, Screen};
use crate::ui::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Credits), enter_credits);
    app.add_systems(
        Update,
        handle_credits_action.run_if(in_state(Screen::Credits)),
    );
    app.register_type::<CreditsAction>();
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Component)]
enum CreditsAction {
    Back,
}

fn enter_credits(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Credits))
        .with_children(|children| {
            children.header("CREDITS");
            children.label("Didn't have time to put in game, sorry.\n Check the game's github page. I'll put it there a couple days after release.");
            children.button("BACK").insert(CreditsAction::Back);
        });
}

fn handle_credits_action(
    mut next_screen: ResMut<NextTransitionedState>,
    mut button_query: InteractionQuery<&CreditsAction>,
) {
    for (interaction, action) in &mut button_query {
        if matches!(interaction, Interaction::Pressed) {
            match action {
                CreditsAction::Back => next_screen.set(Screen::Title),
            }
        }
    }
}

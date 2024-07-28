//! A loading screen during which game assets are loaded.
//! This reduces stuttering, especially for audio on WASM.

use bevy::prelude::*;

use super::{NextTransitionedState, Screen};
use crate::ui::prelude::*;

// todo: use transition
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), enter_loading)
        .add_systems(OnEnter(Screen::Loaded), on_loaded);
}

fn enter_loading(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Loaded))
        .with_children(|children| {
            children.label("Loading...");
        });
}

fn on_loaded(mut next_screen: ResMut<NextTransitionedState>) {
    next_screen.set(Screen::Title);
}

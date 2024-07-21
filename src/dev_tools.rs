//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{dev_tools::states::log_transitions, prelude::*};
use bevy_ecs_ldtk::LevelSelection;

use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    // Print state transitions in dev builds
    app.add_systems(Update, log_transitions::<Screen>)
        .add_systems(Update, process_change_lvl_input);
}

fn process_change_lvl_input(input: Res<ButtonInput<KeyCode>>, mut lvl: ResMut<LevelSelection>) {
    if let LevelSelection::Indices(i) = lvl.as_ref() {
        if let Some(new_lvl) = if input.just_pressed(KeyCode::NumpadAdd) {
            Some(i.level + 1)
        } else if input.just_pressed(KeyCode::NumpadSubtract) {
            Some(i.level.saturating_sub(1))
        } else {
            None
        } {
            *lvl = LevelSelection::index(new_lvl);
        }
    }
}

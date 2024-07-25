//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_toggle_active, prelude::*,
};

#[cfg(feature = "dev")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, log_transitions::<Screen>)
        .add_systems(Update, process_score_raise_input)
        .add_plugins(avian2d::debug_render::PhysicsDebugPlugin::default());

    #[cfg(feature = "dev")]
    app.add_plugins(WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::KeyE)));
}

fn process_score_raise_input(_input: Res<ButtonInput<KeyCode>>) {
    // if let Some(_score) = if input.just_pressed(KeyCode::NumpadAdd) {
    //     Some(())
    // } else if input.just_pressed(KeyCode::NumpadSubtract) {
    //     Some(())
    // } else {
    //     None
    // } {
    //     // todo
    // }
}

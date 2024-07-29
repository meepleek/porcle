//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions, input::common_conditions::input_toggle_active, prelude::*,
};

#[cfg(feature = "dev")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::{
    game::{
        ball::{BallSpeed, BALL_BASE_SPEED},
        spawn::paddle::PaddleAmmo,
    },
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, log_transitions::<Screen>)
        .add_systems(Update, process_debug_input);

    // app.add_plugins(avian2d::debug_render::PhysicsDebugPlugin::default());

    #[cfg(feature = "dev")]
    app.add_plugins(
        WorldInspectorPlugin::new().run_if(input_toggle_active(false, MouseButton::Middle)),
    );
}

fn process_debug_input(
    input: Res<ButtonInput<KeyCode>>,
    mut ammo_q: Query<&mut PaddleAmmo>,
    mut ball_speed_q: Query<&mut BallSpeed>,
) {
    if input.pressed(KeyCode::NumpadAdd) {
        for mut ammo in &mut ammo_q {
            ammo.offset(1);
        }
    }
    if input.pressed(KeyCode::Numpad0) {
        for mut ball_speed in &mut ball_speed_q {
            ball_speed.0 = BALL_BASE_SPEED * 3.0;
        }
    }
}

use bevy::prelude::*;

use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(Update, process_input.in_set(AppSet::ProcessInput));
}

fn process_input(_input: Res<ButtonInput<KeyCode>>, mut _cmd: Commands) {
    // todo:
    // let movement_direction =
    //     if input.just_pressed(KeyCode::KeyW) || input.just_pressed(KeyCode::ArrowUp) {
    //         GridCoords::new(0, 1)
    //     } else if input.just_pressed(KeyCode::KeyA) || input.just_pressed(KeyCode::ArrowLeft) {
    //         GridCoords::new(-1, 0)
    //     } else if input.just_pressed(KeyCode::KeyS) || input.just_pressed(KeyCode::ArrowDown) {
    //         GridCoords::new(0, -1)
    //     } else if input.just_pressed(KeyCode::KeyD) || input.just_pressed(KeyCode::ArrowRight) {
    //         GridCoords::new(1, 0)
    //     } else {
    //         return;
    //     };
}

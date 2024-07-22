use bevy::prelude::*;

use crate::AppSet;

use super::{input::CursorCoords, spawn::paddle::PaddleRotation};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (process_input.in_set(AppSet::ProcessInput), rotate_paddle),
    );
}

fn process_input(_input: Res<ButtonInput<KeyCode>>, mut _cmd: Commands) {}

fn rotate_paddle(
    mut rot_q: Query<&mut Transform, With<PaddleRotation>>,
    cursor: Res<CursorCoords>,
) {
    for mut t in rot_q.iter_mut() {
        if let Ok(dir) = Dir2::new(cursor.0) {
            t.rotation = Quat::from_rotation_z(dir.to_angle());
        }
    }
}

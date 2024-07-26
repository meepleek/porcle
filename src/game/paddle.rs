use bevy::prelude::*;
use std::f32::consts::TAU;

use crate::{
    ext::{QuatExt, Vec2Ext},
    AppSet,
};

use super::{
    input::CursorCoords,
    movement::{MoveDirection, MovementPaused},
    spawn::{
        ball::Ball,
        paddle::{PaddleMode, PaddleRotation},
    },
};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (process_input.in_set(AppSet::ProcessInput), rotate_paddle),
    );
}

const PADDLE_REVOLUTION_DURATION_MIN: f32 = 0.35;

fn process_input(
    input: Res<ButtonInput<MouseButton>>,
    mut paddle_mode_q: Query<(&mut PaddleMode, &GlobalTransform)>,
    mut cmd: Commands,
    mut ball_q: Query<&mut MoveDirection, With<Ball>>,
) {
    if input.just_pressed(MouseButton::Right) {
        for (mut pm, paddle_t) in &mut paddle_mode_q {
            *pm = match *pm {
                PaddleMode::Reflect => PaddleMode::Capture,
                PaddleMode::Capture => PaddleMode::Reflect,
                PaddleMode::Captured {
                    shoot_rotation: rotation,
                    ball_e,
                } => {
                    if let Ok(mut move_dir) = ball_q.get_mut(ball_e) {
                        let dir = (Quat::from_rotation_z(rotation.as_radians())
                            * -paddle_t.right())
                        .truncate()
                        .normalize_or_zero();
                        move_dir.0 = dir;
                        cmd.entity(ball_e)
                            .remove_parent_in_place()
                            .remove::<MovementPaused>();
                    }
                    PaddleMode::Reflect
                }
            };
        }
    }
}

fn rotate_paddle(
    mut rot_q: Query<&mut Transform, With<PaddleRotation>>,
    cursor: Res<CursorCoords>,
    time: Res<Time<Real>>,
) {
    for mut t in rot_q.iter_mut() {
        // limit rotation in the very center/deadzone
        let deadzone_radius = 70.0;
        let radius = cursor.0.length();
        // deadzone multiplier with exponential decay
        let deadzone_mult = (radius / deadzone_radius).min(1.).powf(3.0);
        let current_angle = t.rotation.to_rot2();
        let target_angle = cursor.0.to_rot2();
        let max_delta =
            (time.delta_seconds() / PADDLE_REVOLUTION_DURATION_MIN) * TAU * deadzone_mult;
        let target_delta = current_angle.angle_between(target_angle);
        let clamped_angle =
            current_angle * Rot2::radians(target_delta.clamp(-max_delta, max_delta));
        t.rotation = Quat::from_rotation_z(clamped_angle.as_radians());
    }
}

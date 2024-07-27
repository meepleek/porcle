use bevy::prelude::*;
use std::f32::consts::TAU;

use crate::{
    ext::{QuatExt, Vec2Ext},
    AppSet,
};

use super::{
    ball::MaxBallSpeedFactor,
    input::CursorCoords,
    movement::{AccumulatedRotation, MoveDirection, MovementPaused},
    spawn::{
        ball::{Ball, SpawnBall},
        paddle::{PaddleAmmo, PaddleMode, PaddleRotation},
    },
};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            process_input.in_set(AppSet::ProcessInput),
            rotate_paddle,
            apply_cycle_effects,
        ),
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
                        info!(?dir, "found my ball");
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

fn apply_cycle_effects(
    mut rot_q: Query<(&mut PaddleRotation, &AccumulatedRotation)>,
    mut ammo_q: Query<&mut PaddleAmmo>,
    ball_speed_factor: Res<MaxBallSpeedFactor>,
    mut cmd: Commands,
    time: Res<Time>,
) {
    for (mut paddle_rot, angle) in rot_q.iter_mut() {
        if (angle.rotation - paddle_rot.cw_start) <= -720f32.to_radians() {
            // CW (negative angle)
            paddle_rot.reset(angle.rotation);
            cmd.trigger(SpawnBall {
                paddle_e: paddle_rot.paddle_e,
            });
        } else if (angle.rotation - paddle_rot.ccw_start) >= 360f32.to_radians() {
            // CCW (positive angle)
            for mut ammo in &mut ammo_q {
                ammo.0 += ball_speed_factor.ammo_bonus();
            }
            paddle_rot.reset(angle.rotation);
        } else if angle.rotation > paddle_rot.cw_start {
            paddle_rot.cw_start = angle.rotation;
        } else if angle.rotation < paddle_rot.ccw_start {
            paddle_rot.ccw_start = angle.rotation;
        }

        let delta = (paddle_rot.prev_rot - angle.rotation).abs() / time.delta_seconds();
        if delta < 3. {
            // reset if rotation doesn't change for a while
            paddle_rot.timer.tick(time.delta());
            if paddle_rot.timer.just_finished() {
                paddle_rot.reset(angle.rotation);
            }
        } else {
            paddle_rot.timer.reset()
        }

        paddle_rot.prev_rot = angle.rotation;
    }
}

use bevy::prelude::*;
use bevy_tweening::Animator;
use std::f32::consts::TAU;

use crate::{
    AppSet,
    ext::{QuatExt, Vec2Ext},
};

use super::{
    ball::MaxBallSpeedFactor,
    input::{AimDirection, PlayerAction, PlayerInput},
    movement::{AccumulatedRotation, MoveDirection, MovementPaused},
    spawn::{
        ball::{Ball, SpawnBall},
        level::AmmoUi,
        paddle::{Paddle, PaddleAmmo, PaddleMode, PaddleRotation},
    },
    time::{Cooldown, process_cooldown},
    tween::{get_relative_scale_tween, get_relative_sprite_color_anim},
};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            process_input.in_set(AppSet::ProcessInput),
            rotate_paddle,
            apply_cycle_effects,
            process_cooldown::<PaddleMode>,
        ),
    );
}

pub const PADDLE_REVOLUTION_DURATION_MIN: f32 = 0.45;

fn process_input(
    input: PlayerInput,
    mut paddle_mode_q: Query<
        (Entity, &Paddle, &mut PaddleMode, &GlobalTransform),
        Without<Cooldown<PaddleMode>>,
    >,
    mut cmd: Commands,
    mut ball_q: Query<&mut MoveDirection, With<Ball>>,
) {
    // todo: cooldown?
    if input.just_pressed(&PlayerAction::TogglePaddleMode) {
        for (e, paddle, mut pm, paddle_t) in &mut paddle_mode_q {
            *pm = match *pm {
                PaddleMode::Reflect => PaddleMode::Capture,
                PaddleMode::Capture => PaddleMode::Reflect,
                PaddleMode::Captured {
                    shoot_rotation,
                    ball_e,
                } => {
                    if let Ok(mut move_dir) = ball_q.get_mut(ball_e) {
                        let dir = (Quat::from_rotation_z(shoot_rotation.as_radians())
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
            cmd.entity(paddle.reflect_e)
                .try_insert(get_relative_sprite_color_anim(
                    pm.color(),
                    150,
                    Some(EaseFunction::QuadraticOut),
                ));
            cmd.entity(e).try_insert(Cooldown::<PaddleMode>::new(0.15));
        }
    }
}

fn rotate_paddle(
    mut rot_q: Query<&mut Transform, With<PaddleRotation>>,
    aim_dir: Res<AimDirection>,
    time: Res<Time<Real>>,
) {
    for mut t in rot_q.iter_mut() {
        let current_angle = t.rotation.to_rot2();
        let target_angle = aim_dir.0.to_rot2();
        let max_delta = (time.delta_secs() / PADDLE_REVOLUTION_DURATION_MIN) * TAU;
        let target_delta = current_angle.angle_between(target_angle);
        let clamped_angle =
            current_angle * Rot2::radians(target_delta.clamp(-max_delta, max_delta));
        t.rotation = Quat::from_rotation_z(clamped_angle.as_radians());
    }
}

fn apply_cycle_effects(
    mut rot_q: Query<(&mut PaddleRotation, &AccumulatedRotation)>,
    mut ammo_q: Query<&mut PaddleAmmo>,
    ammo_ui_q: Query<Entity, With<AmmoUi>>,
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
                tween_delay_ms: 0,
            });
        } else if (angle.rotation - paddle_rot.ccw_start) >= 360f32.to_radians() {
            // CCW (positive angle)
            for mut ammo in &mut ammo_q {
                ammo.offset(ball_speed_factor.ammo_bonus() as isize);
            }
            for e in &ammo_ui_q {
                cmd.entity(e).try_insert(Animator::new(
                    get_relative_scale_tween(
                        Vec2::splat(1.25).extend(1.),
                        400,
                        Some(EaseFunction::BackOut),
                    )
                    .then(get_relative_scale_tween(
                        Vec3::ONE,
                        200,
                        Some(EaseFunction::QuadraticOut),
                    )),
                ));
            }
            paddle_rot.reset(angle.rotation);
        } else if angle.rotation > paddle_rot.cw_start {
            paddle_rot.cw_start = angle.rotation;
        } else if angle.rotation < paddle_rot.ccw_start {
            paddle_rot.ccw_start = angle.rotation;
        }

        let delta = (paddle_rot.prev_rot - angle.rotation).abs() / time.delta_secs();
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

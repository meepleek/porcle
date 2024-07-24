use avian2d::prelude::*;
// use bevy::color::palettes::tailwind;
use bevy::prelude::*;

use crate::{
    ext::{QuatExt, Vec2Ext},
    AppSet,
};

use super::{
    input::CursorCoords,
    spawn::{
        ball::{Ball, SpawnBall},
        enemy::Enemy,
        paddle::{Paddle, PaddleAmmo, PaddleRotation, PADDLE_RADIUS},
    },
};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            process_input.in_set(AppSet::ProcessInput),
            rotate_paddle,
            reload_balls,
            reflect_ball,
            accumulate_angle,
            apply_velocity,
        ),
    );
}

#[derive(Component, Debug)]
pub struct Velocity(pub Vec2);

pub const BALL_BASE_SPEED: f32 = 250.;

#[derive(Component, Debug)]
pub struct BaseSpeed(pub f32);

fn apply_velocity(mut move_q: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut t, vel) in &mut move_q {
        t.translation += (vel.0 * time.delta_seconds()).extend(0.);
    }
}

fn process_input(_input: Res<ButtonInput<KeyCode>>, mut _cmd: Commands) {}

fn rotate_paddle(
    mut rot_q: Query<&mut Transform, With<PaddleRotation>>,
    cursor: Res<CursorCoords>,
) {
    // todo: limit speed
    for mut t in rot_q.iter_mut() {
        t.rotation = cursor.0.to_quat();
    }
}

fn reload_balls(
    mut rot_q: Query<(&mut PaddleRotation, &AccumulatedRotation)>,
    mut cmd: Commands,
    time: Res<Time>,
    ball_q: Query<&GlobalTransform, With<Ball>>,
) {
    if !ball_q.is_empty()
        && ball_q
            .iter()
            .any(|t| t.translation().length() < PADDLE_RADIUS * 1.1)
    {
        for (mut paddle_rot, angle) in rot_q.iter_mut() {
            paddle_rot.reset(angle.rotation);
        }
        return;
    }

    // todo: limit speed
    for (mut paddle_rot, angle) in rot_q.iter_mut() {
        let min_rot = 355.0f32.to_radians();

        // CW (negative angle)
        if ((angle.rotation - paddle_rot.cw_start) <= -min_rot) ||
            // CCW (positive angle)
            ((angle.rotation - paddle_rot.ccw_start) >= min_rot)
        {
            paddle_rot.reset(angle.rotation);
            cmd.trigger(SpawnBall);
        } else if angle.rotation > paddle_rot.cw_start {
            paddle_rot.cw_start = angle.rotation;
        } else if angle.rotation < paddle_rot.ccw_start {
            paddle_rot.ccw_start = angle.rotation;
        }

        let delta = (paddle_rot.prev_rot - angle.rotation).abs() / time.delta_seconds();
        if delta < 1. {
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

#[derive(Component, Debug, Default)]
pub struct AccumulatedRotation {
    prev: Option<Rot2>,
    rotation: f32,
}

fn accumulate_angle(mut acc_q: Query<(&mut AccumulatedRotation, &Transform), Changed<Transform>>) {
    for (mut acc, t) in &mut acc_q {
        let rot = t.rotation.to_rot2();
        if let Some(prev) = acc.prev {
            acc.rotation += prev.angle_between(rot);
        }
        acc.prev = Some(rot);
    }
}

fn reflect_ball(
    phys_spatial: SpatialQuery,
    mut ball_q: Query<(&GlobalTransform, &mut Ball, &mut Velocity, &mut BaseSpeed)>,
    mut paddle_q: Query<(&mut PaddleAmmo, &GlobalTransform), With<Paddle>>,
    enemy_q: Query<(), With<Enemy>>,
    mut cmd: Commands,
    time: Res<Time>,
    // mut gizmos: Gizmos,
) {
    for (t, mut ball, mut vel, mut speed) in &mut ball_q {
        if vel.0 == Vec2::ZERO {
            // stationary ball
            continue;
        }
        // gizmos.circle_2d(t.translation().truncate(), ball.0, tailwind::AMBER_600);

        for hit in phys_spatial.shape_hits(
            &Collider::circle(ball.radius),
            t.translation().truncate(),
            0.,
            Dir2::new(vel.0).expect("Non zero velocity"),
            (speed.0 * 1.05) * time.delta_seconds(),
            100,
            false,
            SpatialQueryFilter::default(),
        ) {
            let hit_e = hit.entity;
            if let Ok((mut ammo, _paddle_t)) = paddle_q.get_mut(hit_e) {
                if time.elapsed_seconds() < ball.last_reflection_time + 0.1 {
                    // ignore consecutive hits
                    continue;
                }

                speed.0 *= 1.05;
                // let hit_point = paddle_t.transform_point(hit.point1.extend(0.));
                // info!(/*?hit_point,*/ src = ?hit.point1, paddle = ?paddle_t.translation(), "paddle hit");
                // todo: use hit.point1 to determine the angle
                // todo: also never reflect the ball out even when hitting an edge
                vel.0 = hit.normal1 * speed.0;
                ammo.0 += 1;
                ball.last_reflection_time = time.elapsed_seconds();
            } else if enemy_q.contains(hit_e) {
                cmd.entity(hit_e).despawn_recursive();
            }
        }
    }
}

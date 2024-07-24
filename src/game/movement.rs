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
        ball::{Ball, InsideCore, PaddleReflectionCount, SpawnBall},
        enemy::Enemy,
        level::Wall,
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
            balls_inside_core,
            reflect_ball,
            accumulate_angle,
            apply_velocity,
            apply_homing_velocity,
            apply_damping,
        ),
    );
}

#[derive(Component, Debug)]
pub struct Velocity(pub Vec2);

#[derive(Component, Debug)]
pub struct Damping(pub f32);

pub const BALL_BASE_SPEED: f32 = 250.;

#[derive(Component, Debug)]
pub struct BaseSpeed(pub f32);

#[derive(Component, Debug)]
pub struct Homing {
    pub max_distance: f32,
    pub max_factor: f32,
    pub factor_decay: f32,
    pub max_angle: f32,
}

#[derive(Component, Debug)]
pub struct HomingTarget;

fn apply_velocity(
    mut move_q: Query<(&mut Transform, &Velocity), Without<Homing>>,
    time: Res<Time>,
) {
    for (mut t, vel) in &mut move_q {
        t.translation += (vel.0 * time.delta_seconds()).extend(0.);
    }
}

fn apply_homing_velocity(
    mut move_q: Query<(&mut Transform, &mut Velocity, &Homing)>,
    time: Res<Time>,
    target_q: Query<&GlobalTransform, With<HomingTarget>>,
) {
    for (mut homing_t, mut vel, homing) in &mut move_q {
        let dir = vel.0.normalize_or_zero();
        let mut closest_distance = f32::MAX;
        let mut homing_target_dir = None;

        for target_t in target_q.iter() {
            let distance = homing_t.translation.distance(target_t.translation());

            if distance < closest_distance && distance <= homing.max_distance {
                let target_dir = (target_t.translation() - homing_t.translation)
                    .normalize()
                    .truncate();
                let angle = dir.angle_between(target_dir).to_degrees().abs();

                if angle > homing.max_angle {
                    continue;
                }

                closest_distance = distance;
                homing_target_dir = Some(target_dir);
            }
        }

        if let Some(target_dir) = homing_target_dir {
            // Exponential decay to make homing effect stronger
            let distance_factor = (1.0 - (closest_distance / homing.max_distance))
                .powf(homing.factor_decay)
                * homing.max_factor
                * time.delta_seconds();
            let homing_dir =
                (dir * (1.0 - distance_factor) + target_dir * distance_factor).normalize_or_zero();
            let speed = vel.0.length();
            vel.0 = homing_dir * speed;

            // todo: use if rotation is ever needed
            // homing_t.rotation = homing_dir.to_quat();
        }

        homing_t.translation += (vel.0 * time.delta_seconds()).extend(0.);
    }
}

fn apply_damping(
    mut damping_q: Query<(&mut Velocity, &Damping, Option<&mut BaseSpeed>)>,
    time: Res<Time>,
) {
    for (mut vel, damping, speed) in &mut damping_q {
        let mult = 1. - (damping.0 * time.delta_seconds());
        vel.0 *= mult;
        if let Some(mut speed) = speed {
            speed.0 *= mult;
        }
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

fn balls_inside_core(
    mut cmd: Commands,
    ball_q: Query<(Entity, &GlobalTransform, Option<&InsideCore>), With<Ball>>,
) {
    for (e, t, inside) in &ball_q {
        let inside_core = t.translation().length() < PADDLE_RADIUS * 1.1;
        if inside_core && inside.is_none() {
            cmd.entity(e).insert(InsideCore);
            cmd.entity(e).remove::<Damping>();
            cmd.entity(e).remove::<Homing>();
        } else if !inside_core && inside.is_some() {
            cmd.entity(e).remove::<InsideCore>();
            cmd.entity(e).insert(Damping(0.5));
            cmd.entity(e).insert(Homing {
                max_distance: 300.,
                max_factor: 10.,
                factor_decay: 2.0,
                max_angle: 25.,
            });
        }
    }
}

fn reload_balls(
    mut rot_q: Query<(&mut PaddleRotation, &AccumulatedRotation)>,
    mut cmd: Commands,
    time: Res<Time>,
    ball_q: Query<Option<&InsideCore>, With<Ball>>,
) {
    if !ball_q.is_empty() && ball_q.iter().any(|inside| inside.is_some()) {
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
    mut ball_q: Query<(
        &GlobalTransform,
        &mut Ball,
        &mut Velocity,
        &mut BaseSpeed,
        &mut PaddleReflectionCount,
    )>,
    mut paddle_q: Query<(&mut PaddleAmmo, &GlobalTransform), With<Paddle>>,
    enemy_q: Query<(), With<Enemy>>,
    wall_q: Query<(), With<Wall>>,
    mut cmd: Commands,
    time: Res<Time>,
    // mut gizmos: Gizmos,
) {
    for (t, mut ball, mut vel, mut speed, mut paddle_reflection_count) in &mut ball_q {
        if (vel.0 - Vec2::ZERO).length() < f32::EPSILON {
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

                // clamp to min speed in case the ball has come back to core
                speed.0 = (speed.0 * 1.15).max(BALL_BASE_SPEED);
                // let hit_point = paddle_t.transform_point(hit.point1.extend(0.));
                // info!(/*?hit_point,*/ src = ?hit.point1, paddle = ?paddle_t.translation(), "paddle hit");
                // todo: use hit.point1 to determine the angle
                // todo: also never reflect the ball out even when hitting an edge
                vel.0 = hit.normal1 * speed.0;
                paddle_reflection_count.0 += 1;
                ammo.0 += match paddle_reflection_count.0 {
                    0 => 0,
                    1..=2 => 1,
                    3..=5 => 2,
                    _ => 3,
                };
                ball.last_reflection_time = time.elapsed_seconds();
                info!(ammo=?ammo.0, "added ammo");
            } else if wall_q.contains(hit_e) {
                if time.elapsed_seconds() < ball.last_reflection_time + 0.1 {
                    // ignore consecutive hits
                    continue;
                }
                speed.0 *= 0.7;
                let dir = vel.0.normalize_or_zero();
                let reflect = dir - (2.0 * dir.dot(hit.normal1) * hit.normal1);
                vel.0 = reflect * speed.0;
                ball.last_reflection_time = time.elapsed_seconds();
            } else if enemy_q.contains(hit_e) {
                cmd.entity(hit_e).despawn_recursive();

                // todo: try - boost speed on hit
            }
        }
    }
}

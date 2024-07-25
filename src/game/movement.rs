use std::f32::consts::TAU;

use avian2d::prelude::*;
// use bevy::color::palettes::tailwind;
use bevy::prelude::*;
use bevy_trauma_shake::Shakes;
use bevy_tweening::{Animator, EaseFunction};

use crate::{
    ext::{QuatExt, Vec2Ext},
    game::{spawn::paddle::PADDLE_COLL_HEIGHT, tween::get_relative_translation_tween},
    AppSet,
};

use super::{
    input::CursorCoords,
    spawn::{
        ball::{Ball, InsideCore, PaddleReflectionCount, SpawnBall},
        enemy::Enemy,
        level::Wall,
        paddle::{Paddle, PaddleAmmo, PaddleMode, PaddleRotation, PADDLE_RADIUS},
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
            handle_ball_collisions,
            accumulate_angle,
            apply_velocity,
            apply_homing_velocity,
            apply_damping,
            follow,
        ),
    );
}

pub const BALL_BASE_SPEED: f32 = 250.;
pub const PADDLE_REVOLUTION_DURATION_MIN: f32 = 0.35;

#[derive(Component, Debug)]
pub struct Velocity(pub Vec2);

#[derive(Component, Debug)]
pub struct Damping(pub f32);

#[derive(Component, Debug)]
pub struct BaseSpeed(pub f32);

impl BaseSpeed {
    pub fn speed_factor(&self, min: f32, max: f32) -> f32 {
        ((self.0 - min) / max).clamp(0., 1.)
    }
}

#[derive(Component, Debug)]
pub struct Homing {
    pub max_distance: f32,
    pub max_factor: f32,
    pub factor_decay: f32,
    pub max_angle: f32,
}

#[derive(Component, Debug)]
pub struct HomingTarget;

#[derive(Component, Debug)]
pub struct Follow {
    pub offset: Vec2,
    pub entity: Entity,
}

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

fn follow(mut follow_q: Query<(&mut Transform, &Follow)>, followed_q: Query<&GlobalTransform>) {
    for (mut t, follow) in &mut follow_q {
        if let Ok(followed_t) = followed_q.get(follow.entity) {
            t.translation = followed_t.translation() + follow.offset.extend(0.);
        }
    }
}

fn process_input(
    input: Res<ButtonInput<MouseButton>>,
    mut paddle_mode_q: Query<(&mut PaddleMode, &GlobalTransform)>,
    mut cmd: Commands,
    ball_q: Query<&BaseSpeed, With<Ball>>,
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
                    if let Ok(speed) = ball_q.get(ball_e) {
                        let dir = (Quat::from_rotation_z(rotation.as_radians())
                            * -paddle_t.right())
                        .truncate()
                        .normalize_or_zero();
                        cmd.entity(ball_e)
                            .remove_parent_in_place()
                            .insert(Velocity(dir * speed.0));
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

fn handle_ball_collisions(
    phys_spatial: SpatialQuery,
    mut ball_q: Query<(
        Entity,
        &GlobalTransform,
        &mut Transform,
        &mut Ball,
        &mut Velocity,
        &mut BaseSpeed,
        &mut PaddleReflectionCount,
    )>,
    mut paddle_q: Query<(
        Entity,
        &mut PaddleAmmo,
        &GlobalTransform,
        &Paddle,
        &mut PaddleMode,
    )>,
    enemy_q: Query<(), With<Enemy>>,
    wall_q: Query<(), With<Wall>>,
    mut cmd: Commands,
    time: Res<Time>,
    mut shake: Shakes,
) {
    for (
        ball_e,
        ball_t,
        mut ball_local_t,
        mut ball,
        mut vel,
        mut speed,
        mut paddle_reflection_count,
    ) in &mut ball_q
    {
        if (vel.0 - Vec2::ZERO).length() < f32::EPSILON {
            // stationary ball
            continue;
        }
        // gizmos.circle_2d(t.translation().truncate(), ball.0, tailwind::AMBER_600);
        for hit in phys_spatial.shape_hits(
            &Collider::circle(ball.radius),
            ball_t.translation().truncate(),
            0.,
            Dir2::new(vel.0).expect("Non zero velocity"),
            (speed.0 * 1.05) * time.delta_seconds(),
            100,
            false,
            SpatialQueryFilter::default(),
        ) {
            let hit_e = hit.entity;
            if let Ok((paddle_e, mut ammo, paddle_t, paddle, mut paddle_mode)) =
                paddle_q.get_mut(hit_e)
            {
                if let PaddleMode::Captured { .. } = *paddle_mode {
                    continue;
                }

                if time.elapsed_seconds() < ball.last_reflection_time + 0.2 {
                    // ignore consecutive hits
                    continue;
                }

                let hit_point_local = paddle_t
                    .affine()
                    .inverse()
                    .transform_point(hit.point1.extend(0.));
                // limit upper treshold to 1 to account for the collider rounding
                let angle_factor = (hit_point_local.y / (PADDLE_COLL_HEIGHT / 2.)).min(1.0);
                let angle = angle_factor * -30.0;
                debug!(angle_factor, angle, "paddle hit");

                if let PaddleMode::Capture = *paddle_mode {
                    // catching ball
                    *paddle_mode = PaddleMode::Captured {
                        shoot_rotation: Rot2::radians(angle.to_radians()),
                        ball_e,
                    };
                    cmd.entity(ball_e).set_parent(paddle_e).remove::<Velocity>();
                    ball_local_t.translation = paddle_t
                        .affine()
                        .inverse()
                        .transform_point(ball_t.translation());
                } else {
                    // reflecting ball
                    shake.add_trauma(
                        0.15 + 0.15 * speed.speed_factor(BALL_BASE_SPEED, BALL_BASE_SPEED * 2.0),
                    );
                    // clamp to min speed in case the ball has come back to core
                    speed.0 = (speed.0 * 1.15).max(BALL_BASE_SPEED);
                    // aim the ball based on where it landed on the paddle
                    // the further it lands from the center, the greater the reflection angle
                    // if x is positive, then the hit is from outside => this aims the new dir back into the core
                    let rot = Quat::from_rotation_z(angle.to_radians());
                    let new_dir = (rot * -paddle_t.right()).truncate().normalize_or_zero();
                    vel.0 = new_dir * speed.0;

                    // ammo
                    paddle_reflection_count.0 += 1;
                    ammo.0 += match paddle_reflection_count.0 {
                        0 => 0,
                        1..=2 => 1,
                        3..=5 => 2,
                        _ => 3,
                    };
                    ball.last_reflection_time = time.elapsed_seconds();
                    debug!(ammo=?ammo.0, "added ammo");

                    // tween
                    cmd.entity(paddle.mesh_e).insert(Animator::new(
                        get_relative_translation_tween(
                            ((rot / 3.) * Vec3::X) * 50.,
                            60,
                            Some(EaseFunction::SineOut),
                        )
                        .then(get_relative_translation_tween(
                            Vec3::ZERO,
                            110,
                            Some(EaseFunction::BackOut),
                        )),
                    ));
                }
            } else if wall_q.contains(hit_e) {
                if time.elapsed_seconds() < ball.last_reflection_time + 0.1 {
                    // ignore consecutive hits
                    continue;
                }

                shake.add_trauma(
                    0.1 + 0.225 * speed.speed_factor(BALL_BASE_SPEED * 0.5, BALL_BASE_SPEED * 2.0),
                );
                speed.0 *= 0.7;
                let dir = vel.0.normalize_or_zero();
                let reflect = dir - (2.0 * dir.dot(hit.normal1) * hit.normal1);
                vel.0 = reflect * speed.0;
                ball.last_reflection_time = time.elapsed_seconds();
            } else if enemy_q.contains(hit_e) {
                cmd.entity(hit_e).despawn_recursive();
                shake.add_trauma(0.135);

                // todo: try - boost speed on hit
            }
        }
    }
}

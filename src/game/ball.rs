use avian2d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*};
use bevy_enoki::prelude::OneShot;
use bevy_trauma_shake::Shakes;
use bevy_tweening::{Animator, EaseFunction};

use crate::{
    ext::Vec2Ext,
    game::{
        movement::MovementPaused, spawn::paddle::PADDLE_COLL_HEIGHT,
        tween::get_relative_translation_tween,
    },
    WINDOW_SIZE,
};

use super::{
    assets::ParticleAssets,
    movement::{AccumulatedRotation, Damping, Homing, MoveDirection, Speed, Velocity},
    spawn::{
        ball::{Ball, InsideCore, PaddleReflectionCount, SpawnBall},
        enemy::Enemy,
        level::Wall,
        paddle::{Paddle, PaddleAmmo, PaddleMode, PaddleRotation, PADDLE_RADIUS},
    },
    time::Cooldown,
    tween::lerp_color,
};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            reload_balls,
            balls_inside_core,
            handle_ball_collisions,
            color_ball,
        ),
    );
}

pub const BALL_BASE_SPEED: f32 = 250.;

#[derive(Component, Debug)]
struct ShapecastNearestEnemy;

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
            cmd.entity(e).insert(Damping(0.125));
            cmd.entity(e).insert(Homing {
                max_distance: 300.,
                max_factor: 80.,
                factor_decay: 2.0,
                max_angle: 70.,
                speed_mult: Some(BALL_BASE_SPEED..(BALL_BASE_SPEED * 2.)),
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

fn handle_ball_collisions(
    phys_spatial: SpatialQuery,
    mut ball_q: Query<(
        Entity,
        &GlobalTransform,
        &mut Transform,
        &mut Ball,
        &Velocity,
        &mut MoveDirection,
        &mut Speed,
        &mut PaddleReflectionCount,
    )>,
    ball_shapecast_q: Query<
        (),
        (
            With<ShapecastNearestEnemy>,
            Without<Cooldown<MovementPaused>>,
        ),
    >,
    mut paddle_q: Query<(
        Entity,
        &mut PaddleAmmo,
        &GlobalTransform,
        &Paddle,
        &mut PaddleMode,
    )>,
    enemy_q: Query<&GlobalTransform, With<Enemy>>,
    wall_q: Query<(), With<Wall>>,
    mut cmd: Commands,
    time: Res<Time>,
    mut shake: Shakes,
    particles: Res<ParticleAssets>,
) {
    for (
        ball_e,
        ball_t,
        mut ball_local_t,
        mut ball,
        vel,
        mut direction,
        mut speed,
        mut paddle_reflection_count,
    ) in &mut ball_q
    {
        if (vel.velocity() - Vec2::ZERO).length() < f32::EPSILON {
            // stationary ball
            continue;
        }
        for hit in phys_spatial.shape_hits(
            &Collider::circle(ball.radius),
            ball_t.translation().truncate(),
            0.,
            Dir2::new(vel.velocity()).expect("Non zero velocity"),
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
                    cmd.entity(ball_e)
                        .set_parent(paddle_e)
                        .insert(MovementPaused);
                    ball_local_t.translation = paddle_t
                        .affine()
                        .inverse()
                        .transform_point(ball_t.translation());
                } else {
                    // reflecting ball
                    shake.add_trauma(
                        0.15 + 0.15 * speed.speed_factor(BALL_BASE_SPEED, BALL_BASE_SPEED * 2.0),
                    );
                    cmd.spawn((
                        particles.particle_spawner(
                            particles.reflection.clone(),
                            Transform::from_translation(hit.point1.extend(10.))
                                .with_rotation(paddle_t.up().truncate().to_quat()),
                        ),
                        OneShot::Despawn,
                    ));
                    // clamp to min speed in case the ball has come back to core
                    speed.0 = (speed.0 * 1.225).clamp(BALL_BASE_SPEED, BALL_BASE_SPEED * 5.0);
                    // aim the ball based on where it landed on the paddle
                    // the further it lands from the center, the greater the reflection angle
                    // if x is positive, then the hit is from outside => this aims the new dir back into the core
                    let rot = Quat::from_rotation_z(angle.to_radians());
                    let new_dir = (rot * -paddle_t.right()).truncate().normalize_or_zero();
                    direction.0 = new_dir;

                    // ammo
                    paddle_reflection_count.0 += 1;
                    ammo.0 += match paddle_reflection_count.0 {
                        0 => 0,
                        1..=2 => 1,
                        3..=5 => 2,
                        _ => 3,
                    };
                    let cooldown =
                        0.1 + speed.speed_factor(BALL_BASE_SPEED, BALL_BASE_SPEED * 1.5) * 0.2;
                    cmd.entity(ball_e)
                        .insert(MovementPaused::cooldown(cooldown));
                    ball.last_reflection_time = time.elapsed_seconds() + cooldown;
                    debug!(ammo=?ammo.0, "added ammo");

                    // tween
                    cmd.entity(paddle.mesh_e).insert(Animator::new(
                        get_relative_translation_tween(
                            ((rot / 3.) * Vec3::X) * 50.,
                            60,
                            Some(EaseFunction::QuadraticOut),
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

                let speed_factor = speed.speed_factor(BALL_BASE_SPEED * 0.5, BALL_BASE_SPEED * 2.0);

                // shake
                shake.add_trauma(0.1 + 0.225 * speed_factor);

                // freeze movement
                let cooldown = 0.085 + speed_factor * 0.125;
                cmd.entity(ball_e)
                    .insert((MovementPaused::cooldown(cooldown), ShapecastNearestEnemy));
                ball.last_reflection_time = time.elapsed_seconds() + cooldown;

                // todo: need to fix
                // // particles
                // cmd.spawn((
                //     particles.particle_spawner(
                //         particles.reflection.clone(),
                //         Transform::from_translation(hit.point1.extend(10.)).with_rotation(
                //             Quat::from_rotation_z(-90f32.to_radians()) * hit.normal1.to_quat(),
                //         ),
                //     ),
                //     OneShot::Despawn,
                // ));

                speed.0 *= 0.9;
                let dir = vel.velocity().normalize_or_zero();
                let reflect = dir - (2.0 * dir.dot(hit.normal1) * hit.normal1);
                direction.0 = reflect;
            } else if enemy_q.contains(hit_e) {
                cmd.entity(hit_e).despawn_recursive();
                shake.add_trauma(0.15);
                // particles
                cmd.spawn((
                    particles.square_particle_spawner(
                        particles.enemy.clone(),
                        Transform::from_translation(hit.point1.extend(10.)),
                    ),
                    OneShot::Despawn,
                ));
                // freeze
                let speed_factor =
                    speed.speed_factor(BALL_BASE_SPEED * 0.5, BALL_BASE_SPEED * 1.75);
                let cooldown = 0.08 + speed_factor * 0.12;
                cmd.entity(ball_e)
                    .insert((MovementPaused::cooldown(cooldown), ShapecastNearestEnemy));

                // todo: shapecast cone/triangle & try to find lowest angle coupled with nearest - this can still miss, but we don't wanna be doing predictions 'cause we're lazy

                // todo: try - boost speed on hit or maybe actually take a slight speed hit
            }
        }

        if ball_shapecast_q.get(ball_e).is_ok() {
            cmd.entity(ball_e).remove::<ShapecastNearestEnemy>();
            debug!("shapecasting nearest enemy");
            let radius = 170.;
            let origin = ball_t.translation().truncate() + direction.0 * 150.;
            // gizmos.circle_2d(origin, radius, tailwind::AMBER_700);
            for hit in phys_spatial
                .shape_hits(
                    &Collider::circle(radius),
                    origin,
                    0.,
                    Dir2::new(direction.0).expect("Non zero velocity"),
                    (speed.0 * 1.05) * time.delta_seconds(),
                    100,
                    true,
                    SpatialQueryFilter::default(),
                )
                .iter()
            {
                if let Ok(enemy_t) = enemy_q.get(hit.entity) {
                    let enemy_pos = enemy_t.translation();
                    if enemy_pos.abs().max_element() > (WINDOW_SIZE / 2. - 50.) {
                        // outside window
                        continue;
                    }

                    debug!(pos = ?enemy_t.translation(), "nearest enemy");
                    direction.0 = (enemy_t.translation() - ball_t.translation())
                        .truncate()
                        .normalize_or_zero();
                    break;
                }
            }
        }
    }
}

fn color_ball(
    ball_q: Query<(&Handle<ColorMaterial>, &Speed)>,
    mut mats: ResMut<Assets<ColorMaterial>>,
) {
    for (handle, speed) in &ball_q {
        if let Some(mat) = mats.get_mut(handle) {
            mat.color = lerp_color(
                tailwind::RED_400.into(),
                tailwind::AMBER_300.into(),
                speed.speed_factor(BALL_BASE_SPEED * 1.3, BALL_BASE_SPEED * 2.5),
            );
        }
    }
}

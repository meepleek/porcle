use std::cmp::Ordering;

use avian2d::prelude::*;
use bevy::{core_pipeline::bloom::Bloom, prelude::*};
use bevy_enoki::{
    ParticleEffectHandle,
    prelude::{OneShot, ParticleSpawnerState},
};
use bevy_trauma_shake::{ShakeSettings, Shakes};
use bevy_tweening::Animator;

use crate::{
    BLOOM_BASE, GAME_SIZE,
    ext::Vec2Ext,
    game::{
        movement::MovementPaused,
        spawn::paddle::PADDLE_COLL_HEIGHT,
        tween::{get_relative_sprite_color_anim, get_relative_translation_tween},
    },
    math::asymptotic_smoothing_with_delta_time,
    ui::palette::{COL_BALL, COL_BALL_FAST},
};

use super::{
    assets::ParticleAssets,
    movement::{Homing, MoveDirection, Speed, Velocity, speed_factor},
    score::Score,
    spawn::{
        ball::{Ball, InsidePaddleRadius},
        enemy::Enemy,
        level::Wall,
        paddle::{PADDLE_RADIUS, Paddle, PaddleAmmo, PaddleMode},
    },
    time::Cooldown,
    tween::lerp_color,
};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.init_resource::<MaxBallSpeedFactor>().add_systems(
        Update,
        (
            balls_inside_core,
            update_ball_speed,
            handle_ball_collisions,
            color_ball,
            rotate_ball,
            rotate_ball_particles,
            boost_postprocessing_based_on_ball_speed,
            update_ball_speed_factor,
            update_trauma_based_on_ball_speed,
        ),
    );
}

pub const BALL_BASE_SPEED: f32 = 250.;

#[derive(Component, Debug, Deref, DerefMut, Reflect)]
pub struct BallSpeed(pub f32);

impl Default for BallSpeed {
    fn default() -> Self {
        Self(BALL_BASE_SPEED)
    }
}

#[derive(Resource, Debug, Default, Deref, DerefMut)]
pub struct MaxBallSpeedFactor(pub f32);

impl MaxBallSpeedFactor {
    pub fn ammo_bonus(&self) -> usize {
        ((self.0 * 3.0).round() as usize).max(1) * 2
    }
}

#[derive(Component, Debug)]
struct ShapecastNearestEnemy;

fn balls_inside_core(
    mut cmd: Commands,
    ball_q: Query<(Entity, &GlobalTransform, Option<&InsidePaddleRadius>), With<Ball>>,
) {
    for (e, t, inside) in &ball_q {
        let inside_core = t.translation().length() < PADDLE_RADIUS * 1.1;
        if inside_core && inside.is_none() {
            cmd.entity(e).insert(InsidePaddleRadius);
            cmd.entity(e).remove::<Homing>();
        } else if !inside_core && inside.is_some() {
            cmd.entity(e).remove::<InsidePaddleRadius>();
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

fn update_ball_speed(
    mut ball_q: Query<(&GlobalTransform, &mut Speed, &mut BallSpeed), With<Ball>>,
    paddle_mode_q: Query<&PaddleMode>,
    time: Res<Time>,
) {
    let ball_captured = paddle_mode_q
        .iter()
        .any(|pm| matches!(pm, &PaddleMode::Captured { .. }));

    for (_t, mut speed, mut ball_speed) in &mut ball_q {
        if ball_captured {
            // slow down captured ball
            ball_speed.0 =
                (speed.0 - (BALL_BASE_SPEED * time.delta_secs() * 0.4)).max(BALL_BASE_SPEED);
            debug!(speed = speed.0, "captured ball");
        }

        // let dist = t.translation().length();
        // let factor = ((dist - PADDLE_RADIUS) / 120.0).clamp(0., 1.).powf(2.0);
        // speed.0 = ball_speed.0 * (1. + factor * 0.5);
        speed.0 = ball_speed.0;
    }
}

fn handle_ball_collisions(
    phys_spatial: SpatialQuery,
    mut ball_q: Query<(
        Entity,
        &GlobalTransform,
        &mut Ball,
        &Velocity,
        &mut MoveDirection,
        &Speed,
        &mut BallSpeed,
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
    ball_speed_factor: Res<MaxBallSpeedFactor>,
    mut score: ResMut<Score>,
) {
    for (ball_e, ball_t, mut ball, vel, mut direction, speed, mut ball_speed) in &mut ball_q {
        if (vel.velocity() - Vec2::ZERO).length() < f32::EPSILON {
            // stationary ball
            continue;
        }
        for hit in phys_spatial.shape_hits(
            &Collider::circle(ball.radius),
            ball_t.translation().truncate(),
            0.,
            Dir2::new(vel.velocity()).expect("Non zero velocity"),
            100,
            &ShapeCastConfig::from_max_distance((speed.0 * 1.05) * time.delta_secs()),
            &SpatialQueryFilter::default(),
        ) {
            let hit_e = hit.entity;
            if let Ok((paddle_e, mut ammo, paddle_t, paddle, mut paddle_mode)) =
                paddle_q.get_mut(hit_e)
            {
                if let PaddleMode::Captured { .. } = *paddle_mode {
                    continue;
                }

                if time.elapsed_secs() < ball.last_reflection_time + 0.2 {
                    // ignore consecutive hits
                    continue;
                }

                let hit_point_local = paddle_t
                    .affine()
                    .inverse()
                    .transform_point(hit.point1.extend(0.));
                // limit upper treshold to 1 to account for the collider rounding
                let ratio = hit_point_local.y / (PADDLE_COLL_HEIGHT / 2.);
                let angle_factor = ratio
                    .abs()
                    .min(1.0)
                    // exp decay
                    .powf(1.5);
                // aim the ball based on where it landed on the paddle
                // the further it lands from the center, the greater the reflection angle
                // if x is positive, then the hit is from outside => reflect it back outside
                let origit_rot = if hit_point_local.x > 0. { 180. } else { 0. };
                let max_reflection_angle = 20.0;
                let angle = angle_factor
                    * ratio.signum()
                    * max_reflection_angle
                    * hit_point_local.x.signum()
                    + origit_rot;
                debug!(angle_factor, angle, "paddle hit");

                // allow capturing only from the inside of the core
                if matches!(*paddle_mode, PaddleMode::Capture) && hit_point_local.x < 0. {
                    // catching ball
                    *paddle_mode = PaddleMode::Captured {
                        shoot_rotation: Rot2::radians(angle.to_radians()),
                        ball_e,
                    };
                    cmd.entity(ball_e)
                        .set_parent_in_place(paddle_e)
                        .insert(MovementPaused);
                    cmd.entity(paddle.reflect_e)
                        .try_insert(get_relative_sprite_color_anim(
                            paddle_mode.color(),
                            150,
                            Some(EaseFunction::QuadraticOut),
                        ));
                } else {
                    // reflecting ball
                    shake.add_trauma(
                        0.15 + 0.15 * speed.speed_factor(BALL_BASE_SPEED, BALL_BASE_SPEED * 2.0),
                    );
                    cmd.spawn((
                        particles.circle_particle_spawner(),
                        ParticleEffectHandle(particles.reflection.clone_weak()),
                        Transform::from_translation(hit.point1.extend(10.))
                            .with_rotation(paddle_t.up().truncate().to_quat()),
                        OneShot::Despawn,
                    ));
                    // clamp to min speed in case the ball has come back to core
                    ball_speed.0 = (speed.0 * 1.225).clamp(BALL_BASE_SPEED, BALL_BASE_SPEED * 5.0);
                    let rot = Quat::from_rotation_z(angle.to_radians());
                    let new_dir = (rot * -paddle_t.right()).truncate().normalize_or_zero();
                    direction.0 = new_dir;

                    // ammo
                    ammo.offset(ball_speed_factor.ammo_bonus() as isize);
                    let cooldown =
                        0.1 + speed.speed_factor(BALL_BASE_SPEED, BALL_BASE_SPEED * 1.5) * 0.2;
                    cmd.entity(ball_e)
                        .insert(MovementPaused::cooldown(cooldown));
                    ball.last_reflection_time = time.elapsed_secs() + cooldown;

                    // tween
                    cmd.entity(paddle.sprite_e).insert(Animator::new(
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
                if time.elapsed_secs() < ball.last_reflection_time + 0.1 {
                    // ignore consecutive hits
                    continue;
                }

                let speed_factor = speed.speed_factor(BALL_BASE_SPEED * 0.5, BALL_BASE_SPEED * 2.0);

                // shake
                shake.add_trauma(0.2 + 0.125 * speed_factor);

                // freeze movement
                let cooldown = 0.085 + speed_factor * 0.125;
                cmd.entity(ball_e)
                    .insert((MovementPaused::cooldown(cooldown), ShapecastNearestEnemy));
                ball.last_reflection_time = time.elapsed_secs() + cooldown;

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

                ball_speed.0 *= 0.9;
                let dir = vel.velocity().normalize_or_zero();
                let reflect = dir - (2.0 * dir.dot(hit.normal1) * hit.normal1);
                direction.0 = reflect;
            } else if enemy_q.contains(hit_e) {
                if let Some((_, _, _, _, paddle_mode, ..)) = paddle_q.iter().next() {
                    if matches!(paddle_mode, PaddleMode::Captured { .. }) {
                        continue;
                    }
                }

                cmd.entity(hit_e).despawn_recursive();
                shake.add_trauma(0.15);
                // particles
                cmd.spawn((
                    particles.square_particle_spawner(),
                    ParticleEffectHandle(particles.enemy.clone_weak()),
                    Transform::from_translation(hit.point1.extend(10.)),
                    OneShot::Despawn,
                ));
                // freeze
                let speed_factor =
                    speed.speed_factor(BALL_BASE_SPEED * 0.5, BALL_BASE_SPEED * 1.75);
                let cooldown = 0.08 + speed_factor * 0.06;
                cmd.entity(ball_e)
                    .insert((MovementPaused::cooldown(cooldown), ShapecastNearestEnemy));
                score.0 += 1;
            }
        }

        if ball_shapecast_q.get(ball_e).is_ok() {
            cmd.entity(ball_e).remove::<ShapecastNearestEnemy>();
            debug!("shapecasting nearest enemy");
            let radius = 170.;
            let origin = ball_t.translation().truncate() + direction.0 * 150.;
            for hit in phys_spatial
                .shape_hits(
                    &Collider::circle(radius),
                    origin,
                    0.,
                    Dir2::new(direction.0).expect("Non zero velocity"),
                    100,
                    &ShapeCastConfig {
                        max_distance: (speed.0 * 1.05) * time.delta_secs(),
                        ignore_origin_penetration: true,
                        ..default()
                    },
                    &SpatialQueryFilter::default(),
                )
                .iter()
            {
                if let Ok(enemy_t) = enemy_q.get(hit.entity) {
                    let enemy_pos = enemy_t.translation();
                    if enemy_pos.abs().max_element() > (GAME_SIZE / 2. - 50.) {
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
    ball_q: Query<&Ball>,
    mut sprite_q: Query<&mut Sprite>,
    factor: Res<MaxBallSpeedFactor>,
) {
    for ball in &ball_q {
        if let Ok(mut sprite) = sprite_q.get_mut(ball.sprite_e) {
            sprite.color = lerp_color(COL_BALL, COL_BALL_FAST, factor.0);
        }
    }
}

fn update_ball_speed_factor(
    ball_q: Query<&BallSpeed, With<Ball>>,
    mut factor: ResMut<MaxBallSpeedFactor>,
    time: Res<Time>,
) {
    factor.0 = asymptotic_smoothing_with_delta_time(
        factor.0,
        ball_q
            .iter()
            .map(|speed| speed_factor(speed.0, BALL_BASE_SPEED * 1.3, BALL_BASE_SPEED * 2.5))
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
            .unwrap_or_default(),
        0.1,
        time.delta_secs(),
    );
}

fn boost_postprocessing_based_on_ball_speed(
    factor: Res<MaxBallSpeedFactor>,
    mut bloom_q: Query<&mut Bloom>,
) {
    for mut bloom in &mut bloom_q {
        bloom.intensity = BLOOM_BASE + 0.175 * factor.0;
    }
}

fn update_trauma_based_on_ball_speed(
    factor: Res<MaxBallSpeedFactor>,
    mut shake_q: Query<&mut ShakeSettings>,
) {
    for mut shake in &mut shake_q {
        shake.decay_per_second = 0.8 + 0.35 * factor.0;
        shake.amplitude = 35.0 - 10. * factor.0;
    }
}

fn rotate_ball(
    ball_q: Query<&Ball>,
    mut trans_q: Query<&mut Transform>,
    factor: Res<MaxBallSpeedFactor>,
    time: Res<Time>,
) {
    let base_speed = -480f32.to_radians();
    let factor = 1.0 + factor.0 * 1.5;
    for ball in &ball_q {
        if let Ok(mut t) = trans_q.get_mut(ball.sprite_e) {
            t.rotate_z(base_speed * factor * time.delta_secs());
        }
    }
}

fn rotate_ball_particles(
    ball_q: Query<(Entity, &Ball, &MoveDirection)>,
    ball_paused_q: Query<
        (),
        (
            With<Ball>,
            Or<(With<MovementPaused>, With<Cooldown<MovementPaused>>)>,
        ),
    >,
    mut particles_q: Query<(&mut Transform, &mut ParticleSpawnerState)>,
    factor: Res<MaxBallSpeedFactor>,
) {
    // todo: properly use change detection
    for (e, ball, dir) in &ball_q {
        if let Ok((mut t, mut particles)) = particles_q.get_mut(ball.particles_e) {
            t.rotation = dir.0.to_quat();
            particles.active = !ball_paused_q.contains(e) && factor.0 > 0.25;
        }
    }
}

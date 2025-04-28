use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enoki::prelude::*;
use bevy_trauma_shake::Shakes;
use bevy_tweening::{Animator, Delay, EaseFunction};
use rand::thread_rng;
use std::time::Duration;

use crate::{
    ext::{RandExt, Vec2Ext},
    game::{spawn::projectile::SpawnProjectile, tween::get_relative_scale_anim},
    ui::palette::COL_ENEMY_FLASH,
};

use super::{
    assets::ParticleAssets,
    ball::MaxBallSpeedFactor,
    input::{PlayerAction, PlayerInput},
    movement::{Damping, Impulse, MoveDirection, Speed, Velocity},
    spawn::{
        enemy::{Enemy, Shielded},
        level::Health,
        paddle::{Paddle, PaddleAmmo},
        projectile::Projectile,
    },
    time::{Cooldown, process_cooldown},
    tween::{
        DespawnOnTweenCompleted, delay_tween, get_relative_sprite_color_tween,
        get_relative_translation_tween,
    },
};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            fire_gun,
            handle_collisions,
            process_cooldown::<NoAmmoShake>,
            process_cooldown::<PaddleAmmo>,
        ),
    );
}

struct NoAmmoShake;

fn fire_gun(
    mut ammo_q: Query<
        (
            Entity,
            &Paddle,
            &mut PaddleAmmo,
            &GlobalTransform,
            Option<&Cooldown<NoAmmoShake>>,
        ),
        Without<Cooldown<PaddleAmmo>>,
    >,
    input: PlayerInput,
    mut cmd: Commands,
    mut shake: Shakes,
    particles: Res<ParticleAssets>,
    ball_speed_factor: Res<MaxBallSpeedFactor>,
) {
    if input.pressed(&PlayerAction::Shoot) {
        for (e, paddle, mut ammo, t, cooldown) in &mut ammo_q {
            if ammo.ammo() > 0 {
                let mut rng = thread_rng();
                let accuracy = rng.rotation_range_degrees(4.5);
                let dir = Dir2::new(accuracy * t.right().truncate()).unwrap();
                let rot = (accuracy * t.up().truncate()).to_quat();
                cmd.trigger(SpawnProjectile {
                    dir,
                    transform: Transform::from_translation(
                        t.translation() + (rot * (-Vec3::Y * 80.0)),
                    )
                    .with_rotation(rot),
                });
                ammo.offset(-1);
                shake.add_trauma(0.165 - 0.08 * ball_speed_factor.0);
                cmd.entity(e).insert(Cooldown::<PaddleAmmo>::new(
                    0.17 - 0.08 * ball_speed_factor.0,
                ));

                // tween
                // barrel
                cmd.entity(paddle.barrel_e).insert(Animator::new(
                    get_relative_translation_tween(
                        Vec3::Y * -35.,
                        60,
                        Some(EaseFunction::QuadraticOut),
                    )
                    .then(get_relative_translation_tween(
                        Vec3::ZERO,
                        110,
                        Some(EaseFunction::BackOut),
                    )),
                ));
                // paddle
                cmd.entity(paddle.sprite_e).insert(Animator::new(
                    Delay::new(Duration::from_millis(40))
                        .then(get_relative_translation_tween(
                            Vec3::X * -8.,
                            40,
                            Some(EaseFunction::QuadraticOut),
                        ))
                        .then(get_relative_translation_tween(
                            Vec3::ZERO,
                            60,
                            Some(EaseFunction::BackOut),
                        )),
                ));

                let barrel_pos = t.translation() + t.right() * 80.;
                cmd.spawn((
                    particles.circle_particle_spawner(),
                    particles.gun.clone(),
                    Transform::from_translation(barrel_pos)
                        .with_rotation(t.to_scale_rotation_translation().1),
                    OneShot::Despawn,
                ));
            } else if cooldown.is_none() {
                shake.add_trauma(0.4);
                cmd.entity(e).insert(Cooldown::<NoAmmoShake>::new(1.));

                // todo: some blinking UI or smt. to show there's no ammo
            }
        }
    }
}

fn handle_collisions(
    phys_spatial: SpatialQuery,
    ball_q: Query<(
        Entity,
        &GlobalTransform,
        &Projectile,
        &Velocity,
        &MoveDirection,
        &Speed,
    )>,
    mut enemy_q: Query<(
        &GlobalTransform,
        &Enemy,
        &mut Health,
        &mut Impulse,
        Option<&Shielded>,
    )>,
    mut cmd: Commands,
    time: Res<Time>,
    particles: Res<ParticleAssets>,
) {
    for (e, t, projectile, vel, move_dir, speed) in &ball_q {
        if (vel.velocity() - Vec2::ZERO).length() < f32::EPSILON {
            // stationary
            continue;
        }

        for hit in phys_spatial.shape_hits(
            &Collider::rectangle(projectile.size.x, projectile.size.y),
            t.translation().truncate(),
            0.,
            Dir2::new(move_dir.0).expect("Non zero velocity"),
            (speed.0 * 1.05) * time.delta_secs(),
            100,
            false,
            SpatialQueryFilter::default(),
        ) {
            let hit_e = hit.entity;
            if let Ok((enemy_t, enemy, mut enemy_hp, mut impulse, shielded)) =
                enemy_q.get_mut(hit_e)
            {
                cmd.entity(e).remove::<Projectile>().insert(Damping(30.));
                cmd.entity(projectile.mesh_e).insert((
                    get_relative_scale_anim(
                        Vec2::ZERO.extend(1.),
                        80,
                        Some(EaseFunction::QuadraticOut),
                    ),
                    DespawnOnTweenCompleted::Entity(e),
                ));

                if shielded.is_none() {
                    enemy_hp.0 -= 1;
                }

                if enemy_hp.0 == 0 && shielded.is_none() {
                    cmd.entity(hit_e).remove::<Enemy>().insert(Damping(5.));
                    cmd.entity(enemy.sprite_e).insert((
                        get_relative_scale_anim(
                            Vec2::ZERO.extend(1.),
                            150,
                            Some(EaseFunction::BounceIn),
                        ),
                        DespawnOnTweenCompleted::Entity(hit_e),
                    ));
                    cmd.spawn((
                        particles.enemy.clone(),
                        Transform::from_translation(enemy_t.translation()),
                        OneShot::Despawn,
                    ));
                } else {
                    if shielded.is_none() {
                        // flash
                        cmd.entity(enemy.sprite_e).insert(Animator::new(
                            get_relative_sprite_color_tween(
                                COL_ENEMY_FLASH,
                                50,
                                Some(EaseFunction::QuadraticIn),
                            )
                            .then(delay_tween(
                                get_relative_sprite_color_tween(
                                    enemy.color,
                                    50,
                                    Some(EaseFunction::QuadraticOut),
                                ),
                                150,
                            )),
                        ));
                    }
                    // knockback
                    impulse.0 += move_dir.0 * 30.;
                }
            }
        }
    }
}

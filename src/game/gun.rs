use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enoki::prelude::*;
use bevy_trauma_shake::Shakes;
use bevy_tweening::{Animator, Delay, EaseFunction};
use std::time::Duration;

use crate::{
    ext::Vec2Ext,
    game::{spawn::projectile::SpawnProjectile, tween::get_relative_scale_anim},
};

use super::{
    assets::ParticleAssets,
    ball::MaxBallSpeedFactor,
    core::TakeDamage,
    input::{PlayerAction, PlayerInput},
    movement::{Damping, Impulse, MoveDirection, Speed, Velocity},
    spawn::{
        enemy::{Enemy, EnemyGunBarrel, Shielded},
        level::{Core, Health},
        paddle::{Paddle, PaddleAmmo},
        projectile::{Projectile, ProjectileTarget},
    },
    time::{process_cooldown, Cooldown},
    tween::{get_relative_translation_tween, DespawnOnTweenCompleted},
};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            fire_player_gun,
            fire_enemy_gun,
            handle_collisions,
            process_cooldown::<NoAmmoShake>,
            process_cooldown::<PaddleAmmo>,
            process_cooldown::<EnemyGunBarrel>,
        ),
    );
}

struct NoAmmoShake;

fn fire_player_gun(
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
                let dir = Dir2::new(t.right().truncate()).unwrap();
                let barrel_pos = t.translation() + t.right() * 80.;

                cmd.trigger(SpawnProjectile {
                    target: ProjectileTarget::Enemy,
                    position: barrel_pos.truncate(),
                    dir,
                    max_accuracy_spread: 4.5,
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

                cmd.spawn((
                    particles.particle_spawner(
                        particles.gun.clone(),
                        Transform::from_translation(barrel_pos)
                            .with_rotation(t.to_scale_rotation_translation().1),
                    ),
                    OneShot::Despawn,
                ));
            } else if cooldown.is_none() {
                shake.add_trauma(0.4);
                cmd.entity(e).insert(Cooldown::<NoAmmoShake>::new(1.));
            }
        }
    }
}

fn fire_enemy_gun(
    mut enemy_q: Query<
        (Entity, &GlobalTransform, &EnemyGunBarrel),
        Without<Cooldown<EnemyGunBarrel>>,
    >,
    mut cmd: Commands,
    // particles: Res<ParticleAssets>,
) {
    for (barrel_e, t, barrel) in &mut enemy_q {
        if barrel == &EnemyGunBarrel::Inactive {
            continue;
        }

        let dir = Dir2::new(t.up().truncate()).expect("Valid direction");
        let rot = t.right().truncate().to_quat();
        cmd.trigger(SpawnProjectile {
            target: ProjectileTarget::Core,
            position: (t.translation() + (rot * (Vec3::Y * 20.0))).truncate(),
            dir,
            max_accuracy_spread: 2.0,
        });
        cmd.entity(barrel_e)
            .try_insert(Cooldown::<EnemyGunBarrel>::new(2.5));

        // // tween
        // // barrel
        // cmd.entity(barrel_e).insert(Animator::new(
        //     get_relative_translation_tween(Vec3::Y * -35., 60, Some(EaseFunction::QuadraticOut))
        //         .then(get_relative_translation_tween(
        //             Vec3::ZERO,
        //             110,
        //             Some(EaseFunction::BackOut),
        //         )),
        // ));
        // // enemy
        // cmd.entity(enemy.sprite_e).insert(Animator::new(
        //     Delay::new(Duration::from_millis(40))
        //         .then(get_relative_translation_tween(
        //             Vec3::X * -8.,
        //             40,
        //             Some(EaseFunction::QuadraticOut),
        //         ))
        //         .then(get_relative_translation_tween(
        //             Vec3::ZERO,
        //             60,
        //             Some(EaseFunction::BackOut),
        //         )),
        // ));

        // todo
        // let barrel_pos = t.translation() + t.right() * 80.;
        // cmd.spawn((
        //     particles.particle_spawner(
        //         particles.gun.clone(),
        //         Transform::from_translation(barrel_pos)
        //             .with_rotation(t.to_scale_rotation_translation().1),
        //     ),
        //     OneShot::Despawn,
        // ));
    }
}

fn handle_collisions(
    phys_spatial: SpatialQuery,
    projectile_q: Query<(
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
    core_q: Query<(), With<Core>>,
    mut cmd: Commands,
    time: Res<Time>,
    particles: Res<ParticleAssets>,
    mut taken_dmg_w: EventWriter<TakeDamage>,
) {
    for (e, t, projectile, vel, move_dir, speed) in &projectile_q {
        if (vel.velocity() - Vec2::ZERO).length() < f32::EPSILON {
            // stationary
            continue;
        }

        for hit in phys_spatial.shape_hits(
            &Collider::rectangle(projectile.size.x, projectile.size.y),
            t.translation().truncate(),
            0.,
            Dir2::new(move_dir.0).expect("Non zero velocity"),
            (speed.0 * 1.05) * time.delta_seconds(),
            100,
            false,
            SpatialQueryFilter::default(),
        ) {
            let hit_e = hit.entity;
            let mut despawn = false;
            match projectile.target {
                ProjectileTarget::Enemy => {
                    if let Ok((enemy_t, enemy, mut enemy_hp, mut impulse, shielded)) =
                        enemy_q.get_mut(hit_e)
                    {
                        despawn = true;

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
                                particles.square_particle_spawner(
                                    particles.enemy.clone(),
                                    Transform::from_translation(enemy_t.translation()),
                                ),
                                OneShot::Despawn,
                            ));
                        } else {
                            // knockback
                            impulse.0 += move_dir.0 * 30.;
                        }
                    }
                }
                ProjectileTarget::Core => {
                    if core_q.contains(hit_e) {
                        despawn = true;
                        taken_dmg_w.send_default();
                    }
                }
            }

            // todo: if  rmvd
            if despawn {
                cmd.entity(e).remove::<Projectile>().insert(Damping(30.));
                cmd.entity(projectile.sprite_e).insert((
                    get_relative_scale_anim(
                        Vec2::ZERO.extend(1.),
                        80,
                        Some(EaseFunction::QuadraticOut),
                    ),
                    DespawnOnTweenCompleted::Entity(e),
                ));
            }
        }
    }
}

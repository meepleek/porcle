use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enoki::prelude::*;
use bevy_trauma_shake::Shakes;
use bevy_tweening::{Animator, AssetAnimator, Delay, EaseFunction};
use std::time::Duration;

use crate::{
    ext::Vec2Ext, game::spawn::projectile::SpawnProjectile, game::tween::get_relative_scale_anim,
};

use super::{
    assets::ParticleAssets,
    movement::{Damping, Speed, Velocity},
    spawn::{
        enemy::Enemy,
        level::Health,
        paddle::{Paddle, PaddleAmmo},
        projectile::Projectile,
    },
    time::{process_cooldown, Cooldown},
    tween::{
        delay_tween, get_relative_color_material_color_tween, get_relative_translation_tween,
        DespawnOnTweenCompleted,
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
    input: Res<ButtonInput<MouseButton>>,
    mut cmd: Commands,
    mut shake: Shakes,
    particles: Res<ParticleAssets>,
) {
    if input.just_pressed(MouseButton::Left) {
        for (e, paddle, mut ammo, t, cooldown) in &mut ammo_q {
            // todo: cooldown
            if ammo.0 > 0 {
                let dir = Dir2::new(t.right().truncate()).unwrap();
                let rot = t.up().truncate().to_quat();
                cmd.trigger(SpawnProjectile {
                    dir,
                    transform: Transform::from_translation(
                        t.translation() + (rot * (-Vec3::Y * 70.0)),
                    )
                    .with_rotation(rot),
                });
                ammo.0 -= 1;
                shake.add_trauma(0.125);
                // todo: UI for shoot delay
                cmd.entity(e).insert(Cooldown::<PaddleAmmo>::new(0.14));

                // tween
                // barrel
                cmd.entity(paddle.barrel_e).insert(Animator::new(
                    get_relative_translation_tween(
                        Vec3::X * -40.,
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
                cmd.entity(paddle.mesh_e).insert(Animator::new(
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

                // todo: some blinking UI or smt. to show there's no ammo
            }
        }
    }
}

fn handle_collisions(
    phys_spatial: SpatialQuery,
    ball_q: Query<(Entity, &GlobalTransform, &Projectile, &Velocity, &Speed)>,
    mut enemy_q: Query<(&GlobalTransform, &Enemy, &mut Health)>,
    mut cmd: Commands,
    time: Res<Time>,
    particles: Res<ParticleAssets>,
) {
    for (e, t, projectile, vel, speed) in &ball_q {
        if (vel.velocity() - Vec2::ZERO).length() < f32::EPSILON {
            // stationary
            continue;
        }

        for hit in phys_spatial.shape_hits(
            &Collider::rectangle(projectile.size.x, projectile.size.y),
            t.translation().truncate(),
            0.,
            Dir2::new(vel.velocity()).expect("Non zero velocity"),
            (speed.0 * 1.05) * time.delta_seconds(),
            100,
            false,
            SpatialQueryFilter::default(),
        ) {
            let hit_e = hit.entity;
            if let Ok((enemy_t, enemy, mut enemy_hp)) = enemy_q.get_mut(hit.entity) {
                cmd.entity(e).remove::<Projectile>().insert(Damping(30.));
                cmd.entity(projectile.mesh_e).insert((
                    get_relative_scale_anim(
                        Vec2::ZERO.extend(1.),
                        80,
                        Some(EaseFunction::QuadraticOut),
                    ),
                    DespawnOnTweenCompleted::Entity(e),
                ));
                enemy_hp.0 -= 1;
                if enemy_hp.0 == 0 {
                    cmd.entity(hit_e).remove::<Enemy>().insert(Damping(5.));
                    cmd.entity(enemy.mesh_e).insert((
                        get_relative_scale_anim(
                            Vec2::ZERO.extend(1.),
                            150,
                            Some(EaseFunction::BounceIn),
                        ),
                        DespawnOnTweenCompleted::Entity(hit_e),
                    ));
                    cmd.spawn((
                        particles.particle_spawner(
                            particles.enemy.clone(),
                            Transform::from_translation(enemy_t.translation()),
                        ),
                        OneShot::Despawn,
                    ));
                } else {
                    // flash
                    cmd.entity(enemy.mesh_e).insert(AssetAnimator::new(
                        get_relative_color_material_color_tween(
                            Color::WHITE,
                            50,
                            Some(EaseFunction::QuadraticIn),
                        )
                        .then(delay_tween(
                            get_relative_color_material_color_tween(
                                enemy.color,
                                50,
                                Some(EaseFunction::QuadraticOut),
                            ),
                            150,
                        )),
                    ));
                    // knockback
                }
            }
        }
    }
}

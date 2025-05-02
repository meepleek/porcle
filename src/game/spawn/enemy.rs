use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enoki::{ParticleEffectHandle, prelude::OneShot};
use bevy_tweening::Animator;
use rand::{distributions::WeightedIndex, prelude::*};
use tiny_bail::{c, or_continue};

use crate::{
    GAME_SIZE,
    game::{
        assets::{ParticleAssets, SpriteAssets},
        movement::{Damping, HomingTarget, MoveDirection, Speed, SpeedMultiplier},
        score::Score,
        tween::{
            DespawnOnTweenCompleted, delay_tween, get_relative_scale_anim,
            get_relative_sprite_color_tween,
        },
    },
    math::inverse_lerp_clamped,
    screen::Screen,
    theme::palette::{COL_ENEMY, COL_ENEMY_FLASH},
};

use super::{level::Health, paddle::PADDLE_RADIUS};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_enemy);
    app.add_event::<DespawnEnemy>()
        .add_systems(Last, despawn_enemy)
        .add_systems(
            Update,
            (spawner, enemy_flash_on_hit, slow_down_near_core).run_if(in_state(Screen::Game)),
        );
}

// todo: try to generalise this - it's pretty similar to ProjectileDespawn
#[derive(Event, Debug)]
pub struct DespawnEnemy(pub Entity);

#[derive(Event, Debug)]
pub struct SpawnEnemy {
    pub kind: EnemyKind,
    pub position: Vec2,
}

#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub sprite_e: Entity,
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub enum EnemyGunBarrel {
    Inactive,
    Active,
}

#[derive(Component, Debug, Clone)]
pub struct Shielded;

#[derive(Component, Debug, Clone)]
pub struct StopNearCore(f32);

#[derive(Debug, Clone, Copy)]
pub enum EnemyKind {
    Creepinek,
    Shieldy,
    BigBoi,
    BangBang,
    ShieldedBang,
}

impl EnemyKind {
    fn base_speed(&self) -> f32 {
        match self {
            EnemyKind::Creepinek => 35.,
            EnemyKind::Shieldy => 20.,
            EnemyKind::BigBoi => 15.,
            EnemyKind::BangBang => 30.,
            EnemyKind::ShieldedBang => 15.,
        }
    }

    fn base_time(&self) -> f32 {
        match self {
            EnemyKind::Creepinek => 2.0,
            EnemyKind::Shieldy => 3.0,
            EnemyKind::BigBoi => 4.5,
            EnemyKind::BangBang => 3.,
            EnemyKind::ShieldedBang => 4.5,
        }
    }
}

fn spawner(mut cmd: Commands, mut next_timer: Local<Timer>, time: Res<Time>, score: Res<Score>) {
    next_timer.tick(time.delta());

    if next_timer.just_finished() {
        let mut rng = thread_rng();
        let spawn_dist = (2.0 * (GAME_SIZE / 2.0).powi(2)).sqrt() + 100.;

        let spawnable_kinds = [
            EnemyKind::Creepinek,
            EnemyKind::Shieldy,
            EnemyKind::BangBang,
            EnemyKind::BigBoi,
            EnemyKind::ShieldedBang,
        ];
        let weights = WeightedIndex::new(match score.0 {
            // 0..=2 => [0, 0, 1, 0, 0],
            0..=2 => [1, 0, 0, 0, 0],
            3..=10 => [5, 2, 0, 0, 0],
            11..=22 => [4, 1, 1, 0, 0],
            23.. => [4, 1, 1, 1, 0],
            // 23..=36 => [4, 1, 1, 1, 0],
            // 37..=50 => [4, 1, 1, 1, 1],
            // 51..=65 => [3, 1, 1, 1, 1],
            // 66.. => [3, 2, 2, 1, 1],
        })
        .expect("Create weighted index");

        let kind = spawnable_kinds[weights.sample(&mut rng)];
        cmd.trigger(SpawnEnemy {
            kind,
            position: match kind {
                EnemyKind::Creepinek | EnemyKind::Shieldy | EnemyKind::BigBoi => {
                    (Rot2::degrees(rng.gen_range(-360.0..360.0)) * Vec2::X).normalize() * spawn_dist
                }
                EnemyKind::BangBang | EnemyKind::ShieldedBang => {
                    let base_angle = Rot2::degrees(rng.gen_range(30.0..60.0));
                    let angle = base_angle * Rot2::degrees(90.0 * (rng.gen_range(0..=3) as f32));
                    (angle * Vec2::X).normalize() * spawn_dist
                }
            },
        });
        // todo: balance the spawn rates
        let time_mult_range = match score.0 {
            0..=1 => 2.5..3.0,
            2..=5 => 1.5..2.0,
            6..=15 => 1.2..1.5,
            16..=30 => 0.9..1.2,
            31..=50 => 0.7..1.0,
            51..=70 => 0.5..0.8,
            71..=90 => 0.4..0.7,
            91.. => 0.3..0.5,
        };
        next_timer.set_duration(Duration::from_secs_f32(
            kind.base_time() * rng.gen_range(time_mult_range),
        ));
        next_timer.reset();
    }
}

fn spawn_enemy(trigger: Trigger<SpawnEnemy>, mut cmd: Commands, sprites: Res<SpriteAssets>) {
    let mut rng = thread_rng();

    let ev = trigger.event();
    let speed = rng.gen_range(ev.kind.base_speed()..(ev.kind.base_speed() * 1.5));
    // let speed = rng.gen_range(ev.kind.base_speed()..(ev.kind.base_speed() * 1.5)) * 5.;

    match ev.kind {
        EnemyKind::Creepinek => {
            let size = 45.;
            let a = Vec2::Y * size;
            let b = Vec2::new(-size, -size);
            let c = Vec2::new(size, -size);

            let mesh_e = cmd
                .spawn(Sprite {
                    image: sprites.enemy_creepinek.clone_weak(),
                    color: COL_ENEMY,
                    ..default()
                })
                .id();

            cmd.spawn((
                Name::new("creepinek"),
                Transform::from_translation(ev.position.extend(0.1)).with_rotation(
                    Quat::from_rotation_z(ev.position.to_angle() + 90f32.to_radians()),
                ),
                Visibility::default(),
                Collider::triangle(a, b, c),
                MoveDirection(-ev.position.normalize_or_zero()),
                Speed(speed),
                HomingTarget,
                Enemy { sprite_e: mesh_e },
                Health(3),
                StateScoped(Screen::Game),
            ))
            .add_child(mesh_e);
        }
        EnemyKind::Shieldy => {
            let mesh_e = cmd
                .spawn(Sprite {
                    image: sprites.enemy_creepy_shield.clone_weak(),
                    color: COL_ENEMY,
                    ..default()
                })
                .id();

            cmd.spawn((
                Name::new("shieldy"),
                Transform::from_translation(ev.position.extend(0.1)).with_rotation(
                    Quat::from_rotation_z(ev.position.to_angle() + 90f32.to_radians()),
                ),
                Visibility::default(),
                Collider::ellipse(75., 60.),
                MoveDirection(-ev.position.normalize_or_zero()),
                Speed(speed),
                HomingTarget,
                Enemy { sprite_e: mesh_e },
                Health(3),
                Shielded,
                StateScoped(Screen::Game),
            ))
            .add_child(mesh_e);
        }
        EnemyKind::BigBoi => {
            let size = 95.;
            let a = Vec2::Y * (size - 15.);
            let b = Vec2::new(-size, -size + 10.);
            let c = Vec2::new(size, -size + 10.);

            let sprite_e = cmd
                .spawn(Sprite {
                    image: sprites.enemy_big_boi.clone_weak(),
                    color: COL_ENEMY,
                    ..default()
                })
                .id();

            cmd.spawn((
                Name::new("big_boi"),
                Transform::from_translation(ev.position.extend(0.1)).with_rotation(
                    Quat::from_rotation_z(ev.position.to_angle() + 90f32.to_radians()),
                ),
                Visibility::default(),
                Collider::triangle(a, b, c),
                MoveDirection(-ev.position.normalize_or_zero()),
                Speed(speed),
                HomingTarget,
                Enemy { sprite_e },
                Health(8),
                StateScoped(Screen::Game),
            ))
            .add_child(sprite_e);
        }
        EnemyKind::BangBang => {
            let size = 50.;
            let a = Vec2::Y * (size + 30.);
            let b = Vec2::new(-size, -size);
            let c = Vec2::new(size, -size);

            let sprite_e = cmd
                .spawn(Sprite {
                    image: sprites.enemy_bang.clone(),
                    color: COL_ENEMY,
                    ..default()
                })
                .id();

            let barrel_e = cmd
                .spawn((
                    Sprite {
                        image: sprites.enemy_bang_barrel.clone(),
                        color: COL_ENEMY,
                        ..default()
                    },
                    Transform::from_translation(Vec3::Y * (size + 10.)),
                    EnemyGunBarrel::Inactive,
                ))
                .id();

            cmd.spawn((
                Name::new("bang_bang"),
                Transform::from_translation(ev.position.extend(0.1)).with_rotation(
                    Quat::from_rotation_z(ev.position.to_angle() + 90f32.to_radians()),
                ),
                Visibility::default(),
                Collider::triangle(a, b, c),
                MoveDirection(-ev.position.normalize_or_zero()),
                Speed(speed),
                HomingTarget,
                Enemy { sprite_e },
                Health(5),
                StopNearCore(rng.gen_range((PADDLE_RADIUS * 2.0)..(PADDLE_RADIUS * 2.4))),
                SpeedMultiplier::default(),
                StateScoped(Screen::Game),
            ))
            .add_child(sprite_e)
            .add_child(barrel_e);
        }
        EnemyKind::ShieldedBang => todo!(),
    }
}

// todo: extract to template
fn enemy_flash_on_hit(
    enemy_q: Query<(Entity, &Health), (Changed<Health>, With<Enemy>)>,
    child_q: Query<&Children>,
    sprite_q: Query<&Sprite>,
    mut cmd: Commands,
) {
    for (enemy_e, hp) in &enemy_q {
        if hp.0 > 0 {
            for child_e in child_q.iter_descendants(enemy_e) {
                if sprite_q.contains(child_e) {
                    cmd.entity(child_e).try_insert(Animator::new(
                        get_relative_sprite_color_tween(
                            COL_ENEMY_FLASH,
                            50,
                            Some(EaseFunction::QuadraticIn),
                        )
                        .then(delay_tween(
                            get_relative_sprite_color_tween(
                                COL_ENEMY,
                                50,
                                Some(EaseFunction::QuadraticOut),
                            ),
                            150,
                        )),
                    ));
                }
            }
        }
    }
}

fn slow_down_near_core(
    mut stop_q: Query<(
        Entity,
        &StopNearCore,
        &mut SpeedMultiplier,
        &GlobalTransform,
    )>,
    child_q: Query<&Children>,
    mut barrel_q: Query<&mut EnemyGunBarrel>,
) {
    for (e, stop, mut speed_mult, t) in &mut stop_q {
        let core_dist = t.translation().length();
        let prev_speed_mult = speed_mult.0;
        speed_mult.0 = inverse_lerp_clamped(stop.0, stop.0 + 50.0, core_dist).powf(1.7);
        let cutoff = 0.05;
        let has_stopped = speed_mult.0 <= cutoff && prev_speed_mult > cutoff;
        let has_started = speed_mult.0 > cutoff && prev_speed_mult <= cutoff;

        if has_stopped || has_started {
            for child_e in child_q.iter_descendants(e) {
                let mut barrel = c!(barrel_q.get_mut(child_e));
                *barrel = if has_stopped {
                    EnemyGunBarrel::Active
                } else {
                    EnemyGunBarrel::Inactive
                };
            }
        }
    }
}

fn despawn_enemy(
    mut ev_r: EventReader<DespawnEnemy>,
    mut cmd: Commands,
    enemy_q: Query<(&Enemy, &GlobalTransform)>,
    particles: Res<ParticleAssets>,
) {
    for ev in ev_r.read() {
        let (enemy, t) = or_continue!(enemy_q.get(ev.0));
        cmd.entity(ev.0).remove::<Enemy>().try_insert(Damping(5.));
        cmd.entity(enemy.sprite_e).try_insert((
            get_relative_scale_anim(Vec2::ZERO.extend(1.), 150, Some(EaseFunction::BounceIn)),
            DespawnOnTweenCompleted::Entity(ev.0),
        ));
        cmd.spawn((
            particles.square_particle_spawner(),
            ParticleEffectHandle(particles.enemy.clone_weak()),
            Transform::from_translation(t.translation()),
            OneShot::Despawn,
        ));
    }
}

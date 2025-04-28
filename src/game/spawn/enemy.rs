use std::time::Duration;

use avian2d::prelude::*;
use bevy::prelude::*;
use rand::{distributions::WeightedIndex, prelude::*};

use crate::{
    GAME_SIZE,
    game::{
        assets::SpriteAssets,
        movement::{HomingTarget, MovementBundle},
        score::Score,
    },
    screen::Screen,
    ui::palette::COL_ENEMY,
};

use super::level::Health;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_enemy);
    app.add_systems(Update, spawner.run_if(in_state(Screen::Game)));
}

#[derive(Event, Debug)]
pub struct SpawnEnemy {
    pub kind: EnemyKind,
    pub position: Vec2,
}

#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub sprite_e: Entity,
    pub color: Color,
}

#[derive(Component, Debug, Clone)]
pub struct Shielded;

#[derive(Debug, Clone, Copy)]
pub enum EnemyKind {
    Creepinek,
    Shieldy,
    BigBoi,
}

impl EnemyKind {
    fn base_speed(&self) -> f32 {
        match self {
            EnemyKind::Creepinek => 35.,
            EnemyKind::Shieldy => 20.,
            EnemyKind::BigBoi => 15.,
        }
    }

    fn base_time(&self) -> f32 {
        match self {
            EnemyKind::Creepinek => 2.0,
            EnemyKind::Shieldy => 3.0,
            EnemyKind::BigBoi => 4.5,
        }
    }
}

fn spawner(mut cmd: Commands, mut next_timer: Local<Timer>, time: Res<Time>, score: Res<Score>) {
    next_timer.tick(time.delta());

    if next_timer.just_finished() {
        let mut rng = thread_rng();
        let spawn_dist = (2.0 * (GAME_SIZE / 2.0).powi(2)).sqrt() + 100.;

        let spawnable_kinds = [EnemyKind::Creepinek, EnemyKind::Shieldy, EnemyKind::BigBoi];
        let weights = WeightedIndex::new(match score.0 {
            0..=2 => [1, 0, 0],
            3..=10 => [5, 2, 0],
            11..=25 => [4, 2, 1],
            26.. => [4, 2, 1],
        })
        .expect("Create weighted index");

        let kind = spawnable_kinds[weights.sample(&mut rng)];
        cmd.trigger(SpawnEnemy {
            kind,
            position: (Rot2::degrees(rng.gen_range(-360.0..360.0)) * Vec2::X).normalize()
                * spawn_dist,
        });
        let time_mult_range = match score.0 {
            0..=5 => 1.0..1.3,
            6..=15 => 0.9..1.2,
            16..=30 => 0.8..1.1,
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
                MovementBundle::new(-ev.position.normalize_or_zero(), speed),
                HomingTarget,
                Enemy {
                    sprite_e: mesh_e,
                    color: COL_ENEMY,
                },
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
                MovementBundle::new(-ev.position.normalize_or_zero(), speed),
                HomingTarget,
                Enemy {
                    sprite_e: mesh_e,
                    color: COL_ENEMY,
                },
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
                MovementBundle::new(-ev.position.normalize_or_zero(), speed),
                HomingTarget,
                Enemy {
                    sprite_e,
                    color: COL_ENEMY,
                },
                Health(8),
                StateScoped(Screen::Game),
            ))
            .add_child(sprite_e);
        }
    }
}

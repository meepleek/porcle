use std::time::Duration;

use avian2d::prelude::*;
use bevy::{prelude::*, time::common_conditions::on_timer};
use rand::prelude::*;

use crate::{
    game::{
        assets::SpriteAssets,
        movement::{HomingTarget, MovementBundle},
    },
    screen::Screen,
    ui::palette::COL_ENEMY,
    GAME_SIZE,
};

use super::level::Health;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_enemy);
    app.add_systems(
        Update,
        spawner.run_if(in_state(Screen::Game).and_then(on_timer(Duration::from_millis(1500)))),
    );
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

#[derive(Debug, Clone, Copy)]
pub enum EnemyKind {
    Creepinek,
    CreepyShield,
    BigBoi,
}

impl EnemyKind {
    fn base_speed(&self) -> f32 {
        match self {
            EnemyKind::Creepinek => 25.,
            EnemyKind::CreepyShield => 15.,
            EnemyKind::BigBoi => 10.,
        }
    }
}

fn spawner(mut cmd: Commands) {
    let mut rng = thread_rng();
    let spawn_dist = (2.0 * (GAME_SIZE / 2.0).powi(2)).sqrt() + 100.;

    let spawnable = [EnemyKind::Creepinek, EnemyKind::BigBoi];

    cmd.trigger(SpawnEnemy {
        kind: *spawnable.choose(&mut rng).expect("Kind randomly selected"),
        position: (Rot2::degrees(rng.gen_range(-360.0..360.0)) * Vec2::X).normalize() * spawn_dist,
    });
}

fn spawn_enemy(trigger: Trigger<SpawnEnemy>, mut cmd: Commands, sprites: Res<SpriteAssets>) {
    let mut rng = thread_rng();

    let ev = trigger.event();
    let speed = rng.gen_range(ev.kind.base_speed()..(ev.kind.base_speed() * 1.5));

    match ev.kind {
        EnemyKind::Creepinek => {
            let size = 45.;
            let a = Vec2::Y * size;
            let b = Vec2::new(-size, -size);
            let c = Vec2::new(size, -size);

            let mesh_e = cmd
                .spawn(SpriteBundle {
                    texture: sprites.enemy_creepinek.clone(),
                    sprite: Sprite {
                        color: COL_ENEMY,
                        ..default()
                    },
                    ..default()
                })
                .id();

            cmd.spawn((
                Name::new("Crawler"),
                SpatialBundle::from_transform(
                    Transform::from_translation(ev.position.extend(0.1)).with_rotation(
                        Quat::from_rotation_z(ev.position.to_angle() + 90f32.to_radians()),
                    ),
                ),
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
        EnemyKind::CreepyShield => todo!(),
        EnemyKind::BigBoi => {
            let size = 95.;
            let a = Vec2::Y * (size - 15.);
            let b = Vec2::new(-size, -size + 10.);
            let c = Vec2::new(size, -size + 10.);

            let sprite_e = cmd
                .spawn(SpriteBundle {
                    texture: sprites.enemy_big_boi.clone(),
                    sprite: Sprite {
                        color: COL_ENEMY,
                        ..default()
                    },
                    ..default()
                })
                .id();

            cmd.spawn((
                Name::new("Crawler"),
                SpatialBundle::from_transform(
                    Transform::from_translation(ev.position.extend(0.1)).with_rotation(
                        Quat::from_rotation_z(ev.position.to_angle() + 90f32.to_radians()),
                    ),
                ),
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

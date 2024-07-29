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
    pub speed: f32,
}

#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub sprite_e: Entity,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub enum EnemyKind {
    Creepinek,
    CreepyShield,
    BigBoi,
}

fn spawner(mut cmd: Commands) {
    let mut rng = thread_rng();
    let spawn_dist = (2.0 * (GAME_SIZE / 2.0).powi(2)).sqrt() + 100.;
    cmd.trigger(SpawnEnemy {
        kind: EnemyKind::Creepinek,
        position: (Rot2::degrees(rng.gen_range(-360.0..360.0)) * Vec2::X).normalize() * spawn_dist,
        speed: rng.gen_range(25.0..40.0),
    });
}

fn spawn_enemy(trigger: Trigger<SpawnEnemy>, mut cmd: Commands, sprites: Res<SpriteAssets>) {
    let ev = trigger.event();
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
                MovementBundle::new(-ev.position.normalize_or_zero(), ev.speed),
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
        EnemyKind::BigBoi => todo!(),
    }
}

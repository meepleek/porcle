use avian2d::prelude::*;
use bevy::prelude::*;
use rand::thread_rng;

use crate::{
    ext::{RandExt, Vec2Ext},
    game::{
        assets::SpriteAssets,
        movement::{Damping, MovementBundle},
    },
    screen::Screen,
    ui::palette::{COL_BULLET, COL_ENEMY_PROJECTILE},
};

use super::despawn::DespawnOutOfBounds;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_projectile);
}

#[derive(Event, Debug)]
pub struct SpawnProjectile {
    pub position: Vec2,
    pub dir: Dir2,
    pub target: ProjectileTarget,
    pub max_accuracy_spread: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectileTarget {
    Enemy,
    Core,
}

#[derive(Component, Debug)]
pub struct Projectile {
    pub size: Vec2,
    pub sprite_e: Entity,
    pub target: ProjectileTarget,
}

fn spawn_projectile(
    trigger: Trigger<SpawnProjectile>,
    mut cmd: Commands,
    sprites: Res<SpriteAssets>,
) {
    let mut rng = thread_rng();
    let ev = trigger.event();
    let x = 16.;
    let y = 30.;
    let dir_spread = rng.rotation_range_degrees(ev.max_accuracy_spread);
    let dir = dir_spread * ev.dir;
    let targets_enemy = ev.target == ProjectileTarget::Enemy;
    let sprite_e = cmd
        .spawn(SpriteBundle {
            texture: if targets_enemy {
                sprites.bullet.clone_weak()
            } else {
                sprites.enemy_projectile.clone_weak()
            },
            sprite: Sprite {
                color: if targets_enemy {
                    COL_BULLET
                } else {
                    COL_ENEMY_PROJECTILE
                },
                ..default()
            },
            transform: Transform::from_rotation(Quat::from_rotation_z(180f32.to_radians())),
            ..default()
        })
        .id();
    cmd.spawn((
        Name::new("Projectile"),
        SpatialBundle::from_transform(
            Transform::from_translation(ev.position.extend(0.1))
                .with_rotation(dir.rotate(Vec2::Y).to_quat()),
        ),
        RigidBody::Kinematic,
        if targets_enemy {
            Collider::rectangle(x, y)
        } else {
            Collider::circle(25.)
        },
        MovementBundle::new(dir.as_vec2(), if targets_enemy { 1600. } else { 250. }),
        Damping(if targets_enemy { 0.8 } else { 0.1 }),
        Projectile {
            target: ev.target,
            size: Vec2::new(x, y),
            sprite_e,
        },
        DespawnOutOfBounds,
        StateScoped(Screen::Game),
    ))
    .add_child(sprite_e);
}

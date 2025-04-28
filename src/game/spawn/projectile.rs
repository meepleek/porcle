use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    game::{
        assets::SpriteAssets,
        movement::{Damping, MovementBundle},
    },
    screen::Screen,
    ui::palette::COL_BULLET,
};

use super::despawn::DespawnOutOfBounds;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_projectile);
}

#[derive(Event, Debug)]
pub struct SpawnProjectile {
    pub dir: Dir2,
    pub transform: Transform,
}

#[derive(Component, Debug)]
pub struct Projectile {
    pub size: Vec2,
    pub mesh_e: Entity,
}

fn spawn_projectile(
    trigger: Trigger<SpawnProjectile>,
    mut cmd: Commands,
    sprites: Res<SpriteAssets>,
) {
    let ev = trigger.event();
    let x = 16.;
    let y = 30.;
    let sprite_e = cmd
        .spawn((
            Sprite {
                image: sprites.bullet.clone_weak(),
                color: COL_BULLET,
                ..default()
            },
            Transform::from_rotation(Quat::from_rotation_z(180f32.to_radians())),
        ))
        .id();
    cmd.spawn((
        Name::new("Projectile"),
        SpatialBundle::from_transform(ev.transform),
        RigidBody::Kinematic,
        Collider::rectangle(x, y),
        MovementBundle::new(ev.dir.as_vec2(), 1600.),
        Damping(0.8),
        Projectile {
            size: Vec2::new(x, y),
            mesh_e: sprite_e,
        },
        DespawnOutOfBounds,
        StateScoped(Screen::Game),
    ))
    .add_child(sprite_e);
}

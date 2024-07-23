use avian2d::prelude::*;
use bevy::{
    color::palettes::tailwind,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_projectile);
}

#[derive(Event, Debug)]
pub struct SpawnProjectile {
    pub dir: Dir2,
    pub transform: Transform,
}

#[derive(Component, Debug)]
pub struct Projectile;

fn spawn_projectile(
    trigger: Trigger<SpawnProjectile>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ev = trigger.event();
    let x = 15.;
    let y = 40.;
    cmd.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Rectangle::new(x, y))),
            material: materials.add(ColorMaterial::from_color(tailwind::RED_400)),
            transform: ev.transform,
            ..default()
        },
        RigidBody::Kinematic,
        Collider::rectangle(x, y),
        LinearVelocity(ev.dir.as_vec2() * 1100.),
        Projectile,
        StateScoped(Screen::Game),
    ));
}

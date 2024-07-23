use avian2d::prelude::*;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::{game::movement::AccumulatedRotation, screen::Screen};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_paddle);
}

#[derive(Event, Debug)]
pub struct SpawnPaddle;

#[derive(Component, Debug)]
pub struct Paddle;

#[derive(Component, Debug)]
pub struct PaddleRotation;

fn spawn_paddle(
    _trigger: Trigger<SpawnPaddle>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands
        .spawn((
            SpatialBundle::default(),
            PaddleRotation,
            AccumulatedRotation::default(),
        ))
        .with_children(|b| {
            b.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Capsule2d::new(25.0, 120.0))),
                    material: materials.add(ColorMaterial::from_color(
                        bevy::color::palettes::tailwind::SKY_400,
                    )),
                    transform: Transform::from_xyz(240.0, 0.0, 1.0),
                    ..default()
                },
                RigidBody::Kinematic,
                Collider::capsule(23.0, 130.0),
                Paddle,
                StateScoped(Screen::Game),
            ));
        });
}

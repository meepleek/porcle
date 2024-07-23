use avian2d::prelude::*;
use bevy::{
    color::palettes::tailwind,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::game::movement::{BallSpeed, BALL_BASE_SPEED};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_ball);
}

#[derive(Event, Debug)]
pub struct SpawnBall;

#[derive(Component, Debug)]
pub struct Ball;

fn spawn_ball(
    _trigger: Trigger<SpawnBall>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    cmd.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle { radius: 30.0 })),
            material: materials.add(ColorMaterial::from_color(tailwind::RED_400)),
            transform: Transform::from_xyz(0.0, 0.0, 0.9),
            ..default()
        },
        RigidBody::Kinematic,
        Collider::circle(28.0),
        LinearVelocity(Vec2::X * BALL_BASE_SPEED),
        BallSpeed::default(),
        Ball,
    ));
}

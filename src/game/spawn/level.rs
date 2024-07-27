//! Spawn the main level by triggering other observers.

use avian2d::prelude::*;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::{screen::Screen, WINDOW_SIZE};

use super::{
    ball::SpawnBall,
    paddle::{Paddle, SpawnPaddle, PADDLE_RADIUS},
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level)
        .add_systems(Update, add_ball_to_paddle);
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

#[derive(Component, Debug)]
pub struct Core;

#[derive(Component, Debug)]
pub struct Health(pub u8);

#[derive(Component, Debug)]
pub struct Wall;

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    cmd.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(
                meshes.add(Annulus::new(PADDLE_RADIUS - 10.0, PADDLE_RADIUS + 10.0)),
            ),
            material: materials.add(ColorMaterial::from_color(
                bevy::color::palettes::tailwind::INDIGO_200,
            )),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        Collider::circle(PADDLE_RADIUS),
        RigidBody::Static,
        Core,
        Health(5),
        StateScoped(Screen::Game),
    ));

    cmd.trigger(SpawnPaddle);

    let half_size = WINDOW_SIZE / 2.;

    for (a, b) in [
        (Vec2::new(-1., 1.), Vec2::ONE),
        (Vec2::ONE, Vec2::new(1., -1.)),
        (Vec2::new(1., -1.), Vec2::NEG_ONE),
        (Vec2::NEG_ONE, Vec2::new(-1., 1.)),
    ] {
        cmd.spawn((
            // TransformBundle::default(),
            Collider::segment(a * -half_size, b * half_size),
            Wall,
            StateScoped(Screen::Game),
        ));
    }
}

fn add_ball_to_paddle(paddle_q: Query<Entity, Added<Paddle>>, mut cmd: Commands) {
    for paddle_e in &paddle_q {
        cmd.trigger(SpawnBall { paddle_e });
    }
}

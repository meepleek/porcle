//! Spawn the main level by triggering other observers.

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::screen::Screen;

use super::{
    ball::SpawnBall,
    enemy::{self, SpawnEnemy},
    paddle::SpawnPaddle,
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level);
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Annulus::new(230.0, 250.0))),
            material: materials.add(ColorMaterial::from_color(
                bevy::color::palettes::tailwind::INDIGO_200,
            )),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..default()
        },
        StateScoped(Screen::Game),
    ));

    commands.trigger(SpawnPaddle);
    commands.trigger(SpawnBall);
    commands.trigger(SpawnEnemy {
        enemy: enemy::Enemy::Crawler,
        position: Vec2::new(400., 100.),
    });
}

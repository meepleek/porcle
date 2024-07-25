use std::time::Duration;

use avian2d::prelude::*;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::{game::movement::AccumulatedRotation, screen::Screen};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_paddle);
}

pub const PADDLE_RADIUS: f32 = 240.0;
pub const PADDLE_HEIGHT: f32 = 120.0;
pub const PADDLE_COLL_HEIGHT: f32 = PADDLE_HEIGHT + 10.;

#[derive(Event, Debug)]
pub struct SpawnPaddle;

#[derive(Component, Debug)]
pub struct Paddle;

#[derive(Component, Debug)]
pub struct PaddleRotation {
    pub cw_start: f32,
    pub ccw_start: f32,
    pub timer: Timer,
    pub prev_rot: f32,
}

impl Default for PaddleRotation {
    fn default() -> Self {
        Self {
            cw_start: 0.,
            ccw_start: 0.,
            timer: Timer::new(Duration::from_millis(50), TimerMode::Once),
            prev_rot: 0.,
        }
    }
}

impl PaddleRotation {
    pub fn reset(&mut self, rotation: f32) {
        self.cw_start = rotation;
        self.ccw_start = rotation;
        self.prev_rot = rotation;
        self.timer.reset();
        self.timer.unpause();
    }
}

#[derive(Component, Debug, Default)]
pub struct PaddleAmmo(pub usize);

fn spawn_paddle(
    _trigger: Trigger<SpawnPaddle>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mat = materials.add(ColorMaterial::from_color(
        bevy::color::palettes::tailwind::SKY_400,
    ));

    commands
        .spawn((
            SpatialBundle::default(),
            PaddleRotation::default(),
            AccumulatedRotation::default(),
        ))
        .with_children(|b| {
            b.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Capsule2d::new(25.0, PADDLE_HEIGHT))),
                    material: mat.clone(),
                    transform: Transform::from_xyz(PADDLE_RADIUS, 0.0, 1.0),
                    ..default()
                },
                Collider::capsule(23.0, PADDLE_COLL_HEIGHT),
                Paddle,
                PaddleAmmo::default(),
                StateScoped(Screen::Game),
            ))
            .with_children(|b| {
                b.spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Capsule2d::new(25.0, 10.0))),
                    material: mat.clone(),
                    transform: Transform::from_xyz(15., 0., 0.),
                    ..default()
                });

                b.spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Rectangle::new(25.0, 50.0))),
                    material: mat,
                    transform: Transform::from_xyz(40., 0., 0.)
                        .with_rotation(Quat::from_rotation_z(90f32.to_radians())),
                    ..default()
                });
            });
        });
}

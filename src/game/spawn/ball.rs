use bevy::{
    color::palettes::tailwind,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::{
    game::{
        ball::BALL_BASE_SPEED,
        movement::{MovementBundle, Speed},
    },
    screen::Screen,
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_ball)
        .add_systems(Update, despawn_stationary_balls);
}

pub const BALL_BASE_RADIUS: f32 = 30.;

#[derive(Event, Debug)]
pub struct SpawnBall;

#[derive(Component, Debug)]
pub struct Ball {
    pub radius: f32,
    pub last_reflection_time: f32,
}

#[derive(Component, Debug)]
pub struct PaddleReflectionCount(pub usize);

#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct InsideCore;

impl Default for Ball {
    fn default() -> Self {
        Self {
            radius: BALL_BASE_RADIUS,
            last_reflection_time: 0.,
        }
    }
}

fn spawn_ball(
    _trigger: Trigger<SpawnBall>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ball_q: Query<Entity, With<Ball>>,
) {
    for e in &ball_q {
        cmd.entity(e).despawn_recursive();
    }

    // todo: random
    let dir = Dir2::new(Vec2::X).unwrap();
    // let dir = Dir2::new(-Vec2::Y).unwrap();

    // todo: switch to shapecaster instead?
    // or fix the collision weirdness
    cmd.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(meshes.add(Circle {
                radius: BALL_BASE_RADIUS,
            })),
            material: materials.add(ColorMaterial::from_color(tailwind::RED_400)),
            transform: Transform::from_xyz(0.0, 0.0, 0.9),
            ..default()
        },
        MovementBundle::new(dir.as_vec2(), BALL_BASE_SPEED),
        Ball::default(),
        InsideCore,
        PaddleReflectionCount(0),
        StateScoped(Screen::Game),
    ));
}

fn despawn_stationary_balls(
    mut cmd: Commands,
    ball_q: Query<(Entity, &Speed), (With<Ball>, Without<InsideCore>)>,
) {
    for (e, _) in ball_q.iter().filter(|(_, speed)| speed.0 < 10.) {
        cmd.entity(e).despawn_recursive();
    }
}

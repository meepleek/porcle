use bevy::{
    color::palettes::tailwind,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::{
    ext::Vec2Ext,
    game::{
        ball::BALL_BASE_SPEED,
        movement::{MovementBundle, MovementPaused, Speed},
    },
    screen::Screen,
};

use super::paddle::PaddleMode;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_ball)
        .add_systems(Update, despawn_stationary_balls);
}

pub const BALL_BASE_RADIUS: f32 = 30.;

#[derive(Event, Debug)]
pub struct SpawnBall {
    pub paddle_e: Entity,
}

#[derive(Component, Debug)]
pub struct Ball {
    pub radius: f32,
    pub last_reflection_time: f32,
}

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
    trigger: Trigger<SpawnBall>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ball_q: Query<Entity, With<Ball>>,
    mut paddle_q: Query<(&GlobalTransform, &mut PaddleMode)>,
) {
    for e in &ball_q {
        cmd.entity(e).despawn_recursive();
    }

    let ev = trigger.event();
    if let Ok((paddle_t, mut paddle_mode)) = paddle_q.get_mut(ev.paddle_e) {
        let ball_e = cmd
            .spawn((
                Name::new("Ball"),
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Circle {
                        radius: BALL_BASE_RADIUS,
                    })),
                    material: materials.add(ColorMaterial::from_color(tailwind::RED_400)),
                    transform: Transform::from_xyz(BALL_BASE_RADIUS * -1.8, 0., 0.9),
                    ..default()
                },
                // todo?:
                MovementBundle::new(Vec2::X, BALL_BASE_SPEED),
                MovementPaused,
                Ball::default(),
                InsideCore,
                StateScoped(Screen::Game),
            ))
            .set_parent(ev.paddle_e)
            .id();
        *paddle_mode = PaddleMode::Captured {
            ball_e,
            shoot_rotation: paddle_t.right().truncate().to_rot2(),
        };
    }
}

fn despawn_stationary_balls(
    mut cmd: Commands,
    ball_q: Query<(Entity, &Speed), (With<Ball>, Without<InsideCore>)>,
) {
    for (e, _) in ball_q.iter().filter(|(_, speed)| speed.0 < 10.) {
        cmd.entity(e).despawn_recursive();
    }
}

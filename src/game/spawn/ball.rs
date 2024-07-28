use bevy::{color::palettes::tailwind, prelude::*};
use bevy_tweening::{Animator, EaseFunction};

use crate::{
    game::{
        assets::SpriteAssets,
        ball::{BallSpeed, BALL_BASE_SPEED},
        movement::{MovementBundle, MovementPaused},
        tween::{delay_tween, get_relative_scale_tween},
    },
    screen::Screen,
};

use super::paddle::PaddleMode;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_ball);
}

pub const BALL_BASE_RADIUS: f32 = 40.;

#[derive(Event, Debug)]
pub struct SpawnBall {
    pub paddle_e: Entity,
    pub tween_delay_ms: u64,
}

#[derive(Component, Debug)]
pub struct Ball {
    pub radius: f32,
    pub last_reflection_time: f32,
    pub sprite_e: Entity,
}

#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct InsidePaddleRadius;

impl Ball {
    fn new(sprite_e: Entity) -> Self {
        Self {
            radius: BALL_BASE_RADIUS,
            last_reflection_time: 0.,
            sprite_e,
        }
    }
}

fn spawn_ball(
    trigger: Trigger<SpawnBall>,
    mut cmd: Commands,
    ball_q: Query<Entity, With<Ball>>,
    mut paddle_q: Query<&mut PaddleMode>,
    sprites: Res<SpriteAssets>,
) {
    for e in &ball_q {
        cmd.entity(e).despawn_recursive();
    }

    let ev = trigger.event();
    if let Ok(mut paddle_mode) = paddle_q.get_mut(ev.paddle_e) {
        let sprite_e = cmd
            .spawn((
                Name::new("sprite"),
                SpriteBundle {
                    texture: sprites.ball.clone(),
                    sprite: Sprite {
                        color: tailwind::YELLOW_400.into(),
                        ..default()
                    },
                    transform: Transform::from_scale(Vec3::Z),
                    ..default()
                },
                // todo: delay
                Animator::new(delay_tween(
                    get_relative_scale_tween(Vec3::ONE, 500, Some(EaseFunction::BackOut)),
                    ev.tween_delay_ms,
                )),
            ))
            .id();

        let ball_e = cmd
            .spawn((
                Name::new("Ball"),
                SpatialBundle::from_transform(Transform::from_xyz(
                    BALL_BASE_RADIUS * -1.6,
                    0.,
                    0.9,
                )),
                BallSpeed::default(),
                MovementBundle::new(Vec2::X, BALL_BASE_SPEED),
                MovementPaused,
                Ball::new(sprite_e),
                InsidePaddleRadius,
                StateScoped(Screen::Game),
            ))
            .add_child(sprite_e)
            .set_parent(ev.paddle_e)
            .id();
        *paddle_mode = PaddleMode::Captured {
            ball_e,
            shoot_rotation: Rot2::degrees(0.),
        };
    }
}

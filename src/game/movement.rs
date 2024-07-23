use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{ext::Vec2Ext, AppSet};

use super::{
    input::CursorCoords,
    spawn::{
        ball::Ball,
        enemy::Enemy,
        paddle::{Paddle, PaddleRotation},
    },
};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            process_input.in_set(AppSet::ProcessInput),
            rotate_paddle,
            reflect_ball,
        ),
    );
}

pub const BALL_BASE_SPEED: f32 = 250.;

#[derive(Component, Debug)]
pub struct BallSpeed(f32);

impl Default for BallSpeed {
    fn default() -> Self {
        Self(BALL_BASE_SPEED)
    }
}

fn process_input(_input: Res<ButtonInput<KeyCode>>, mut _cmd: Commands) {}

fn rotate_paddle(
    mut rot_q: Query<&mut Transform, With<PaddleRotation>>,
    cursor: Res<CursorCoords>,
) {
    for mut t in rot_q.iter_mut() {
        t.rotation = cursor.0.to_quat();
    }
}

fn reflect_ball(
    mut coll_q: Query<
        (
            Entity,
            &CollidingEntities,
            &mut LinearVelocity,
            &mut BallSpeed,
        ),
        With<Ball>,
    >,
    paddle_q: Query<(), With<Paddle>>,
    enemy_q: Query<(), With<Enemy>>,
    collisions: Res<Collisions>,
    mut cmd: Commands,
) {
    for (e, colliding, mut vel, mut speed) in &mut coll_q {
        if !colliding.is_empty() {
            let colliding_e = *colliding.0.iter().next().unwrap();
            if paddle_q.contains(colliding_e) {
                if let Some(coll) = collisions.get(e, colliding_e) {
                    if let Some(contact) = coll.manifolds.first() {
                        speed.0 += 5.;
                        vel.0 = contact.normal1 * -speed.0;
                    }
                }
            } else if enemy_q.contains(colliding_e) {
                cmd.entity(colliding_e).despawn_recursive();
            }
        }
    }
}

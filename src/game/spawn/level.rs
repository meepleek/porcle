//! Spawn the main level by triggering other observers.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{GAME_SIZE, screen::Screen};

use super::{
    ball::SpawnBall,
    paddle::{Paddle, SpawnPaddle},
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_level)
        .add_systems(Update, (add_ball_to_paddle,));
}

#[derive(Event, Debug)]
pub struct SpawnLevel;

#[derive(Component, Debug)]
pub struct Core {
    pub gear_entities: Vec<(Entity, bool)>,
    pub clear_mesh_e: Entity,
}

#[derive(Component, Debug)]
pub struct Health(pub u8);

#[derive(Component, Debug)]
pub struct Wall;

#[derive(Component, Debug)]
pub struct RotateWithPaddle {
    pub invert: bool,
    pub offset: Rot2,
    pub multiplier: f32,
}

#[derive(Component, Debug)]
pub struct AmmoUi;

#[derive(Component, Debug)]
pub struct AmmoFill;

fn spawn_level(_trigger: Trigger<SpawnLevel>, mut cmd: Commands) {
    cmd.trigger(SpawnPaddle);

    let half_size = GAME_SIZE / 2.;

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
        cmd.trigger(SpawnBall {
            paddle_e,
            tween_delay_ms: 1400,
        });
    }
}

//! Spawn the main level by triggering other observers.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    ext::QuatExt,
    game::assets::{HandleMap, ParticleAssets, SpriteKey},
    screen::Screen,
    WINDOW_SIZE,
};

use super::{
    ball::SpawnBall,
    paddle::{Paddle, PaddleRotation, SpawnPaddle},
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level)
        .add_systems(Update, (add_ball_to_paddle, rotate_gears));
}

pub const CORE_RADIUS: f32 = 90.0;

#[derive(Event, Debug)]
pub struct SpawnLevel;

#[derive(Component, Debug)]
pub struct Core;

#[derive(Component, Debug)]
pub struct Health(pub u8);

#[derive(Component, Debug)]
pub struct Wall;

#[derive(Component, Debug)]
pub struct Gear {
    even: bool,
    offset: Rot2,
}

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut cmd: Commands,
    sprites: Res<HandleMap<SpriteKey>>,
    particles: Res<ParticleAssets>,
) {
    cmd.spawn((
        SpatialBundle::default(),
        Collider::circle(CORE_RADIUS),
        RigidBody::Static,
        Core,
        Health(5),
        StateScoped(Screen::Game),
    ))
    .with_children(|b| {
        // big gear
        b.spawn((
            Name::new("big_gear"),
            SpriteBundle {
                texture: sprites.get(&SpriteKey::GearBig).unwrap().clone(),
                transform: Transform::from_translation(Vec3::Z * 0.1),
                ..default()
            },
        ));
        // small gear
        let small_gear_count = 8;
        for i in 0..small_gear_count {
            let rot = Rot2::degrees((360f32 / small_gear_count as f32) * i as f32);
            let angle = rot.as_radians() + 18f32.to_radians();
            b.spawn((
                Name::new("small_gear"),
                SpriteBundle {
                    texture: sprites.get(&SpriteKey::GearSmall).unwrap().clone(),
                    transform: Transform::from_translation(((rot * Vec2::X) * 71.).extend(0.1))
                        .with_rotation(Quat::from_rotation_z(angle)),
                    ..default()
                },
                Gear {
                    even: i % 2 == 0,
                    offset: Rot2::radians(angle),
                },
            ));
        }
        //particles
        b.spawn((particles.particle_spawner(particles.core.clone(), Transform::default()),));
    });

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

fn rotate_gears(
    paddle_rot_q: Query<&Transform, With<PaddleRotation>>,
    mut gear_q: Query<(&mut Transform, &Gear), Without<PaddleRotation>>,
) {
    if let Some(paddle_t) = paddle_rot_q.iter().next() {
        for (mut gear_t, gear) in &mut gear_q {
            gear_t.rotation = Quat::from_rotation_z(
                (gear.offset.as_radians() + paddle_t.rotation.z_angle_rad())
                    * (if gear.even { 1. } else { -1. }),
            );
        }
    }
}

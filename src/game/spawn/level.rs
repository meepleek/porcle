//! Spawn the main level by triggering other observers.

use avian2d::prelude::*;
use bevy::{
    color::palettes::tailwind,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::{
    ext::QuatExt,
    game::assets::{HandleMap, ParticleAssets, SpriteKey},
    screen::Screen,
    WINDOW_SIZE,
};

use super::{
    ball::SpawnBall,
    paddle::{Paddle, PaddleAmmo, PaddleRotation, SpawnPaddle},
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level)
        .add_systems(Update, (add_ball_to_paddle, rotate_gears, update_ammo_fill));
}

pub const CORE_RADIUS: f32 = 90.0;
pub const AMMO_FILL_RADIUS: f32 = 34.0;

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

#[derive(Component, Debug)]
pub struct AmmoFill;

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut cmd: Commands,
    sprites: Res<HandleMap<SpriteKey>>,
    particles: Res<ParticleAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    cmd.spawn((
        Name::new("core"),
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
            Name::new("ammo_sprite"),
            SpriteBundle {
                texture: sprites.get(&SpriteKey::Ammo).unwrap().clone(),
                transform: Transform::from_translation(Vec3::Z * 0.3),
                ..default()
            },
        ));

        b.spawn((
            Name::new("ammo_fill"),
            AmmoFill,
            MaterialMesh2dBundle {
                // mesh: Mesh2dHandle(ammo_fill_handle.clone()),
                material: materials.add(ColorMaterial::from_color(tailwind::GREEN_400)),
                transform: Transform::from_translation(Vec3::Z * 0.2)
                    .with_rotation(Quat::from_rotation_z(180f32.to_radians())),
                ..default()
            },
        ));

        b.spawn((
            Name::new("ammo_bg"),
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Circle::new(AMMO_FILL_RADIUS))),
                material: materials.add(ColorMaterial::from_color(tailwind::GRAY_800)),
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

fn update_ammo_fill(
    ammo_q: Query<&PaddleAmmo, Changed<PaddleAmmo>>,
    ammo_fill_q: Query<Entity, With<AmmoFill>>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if let Some(ammo) = ammo_q.iter().next() {
        for e in &ammo_fill_q {
            cmd.entity(e)
                .try_insert(Mesh2dHandle(meshes.add(CircularSegment::from_turns(
                    AMMO_FILL_RADIUS,
                    // not sure why, but the segments fills at 95% already
                    ammo.factor() * 0.95,
                ))));
        }
    }
}

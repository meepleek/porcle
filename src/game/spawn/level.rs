//! Spawn the main level by triggering other observers.

use avian2d::prelude::*;
use bevy::{
    color::palettes::tailwind,
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_tweening::{Animator, EaseFunction};

use crate::{
    game::{
        assets::{ParticleAssets, SpriteAssets},
        tween::{delay_tween, get_relative_scale_tween},
    },
    screen::Screen,
    WINDOW_SIZE,
};

use super::{
    ball::SpawnBall,
    paddle::{Paddle, SpawnPaddle},
};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_level)
        .add_systems(Update, (add_ball_to_paddle,));
}

pub const CORE_RADIUS: f32 = 90.0;
pub const AMMO_FILL_RADIUS: f32 = 34.0;
pub const GEAR_COUNT: u8 = 8;

#[derive(Event, Debug)]
pub struct SpawnLevel;

#[derive(Component, Debug)]
pub struct Core {
    pub gear_entity_ids: Vec<(Entity, bool)>,
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

fn spawn_level(
    _trigger: Trigger<SpawnLevel>,
    mut cmd: Commands,
    sprites: Res<SpriteAssets>,
    particles: Res<ParticleAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // todo: make the gears & center icons lighter/improve contrast with ball
    let cog_entity_ids: Vec<_> = (0..GEAR_COUNT)
        .map(|i| {
            let rot = Rot2::degrees((360f32 / GEAR_COUNT as f32) * i as f32 + 90.);
            let angle = rot.as_radians() + 18f32.to_radians();
            cmd.spawn((
                Name::new("small_gear"),
                SpriteBundle {
                    texture: sprites.gear_small.clone(),
                    transform: Transform::from_translation(((rot * Vec2::X) * 71.).extend(0.1))
                        .with_rotation(Quat::from_rotation_z(angle))
                        .with_scale(Vec2::ZERO.extend(1.)),
                    ..default()
                },
                RotateWithPaddle {
                    invert: i % 2 == 0,
                    offset: Rot2::radians(angle),
                    multiplier: 1.0,
                },
                Animator::new(delay_tween(
                    get_relative_scale_tween(Vec3::ONE, 400, Some(EaseFunction::BackOut)),
                    350 + i as u64 * 100,
                )),
            ))
            .id()
        })
        .collect();

    cmd.spawn((
        Name::new("core"),
        SpatialBundle::default(),
        Collider::circle(CORE_RADIUS),
        RigidBody::Static,
        Core {
            gear_entity_ids: cog_entity_ids
                .iter()
                .cloned()
                .map(|e| (e, true))
                .rev()
                .collect(),
        },
        Health(GEAR_COUNT),
        StateScoped(Screen::Game),
    ))
    .push_children(&cog_entity_ids)
    .with_children(|b| {
        b.spawn((
            SpatialBundle::from_transform(Transform::from_scale(Vec2::ZERO.extend(1.))),
            AmmoUi,
            Animator::new(delay_tween(
                get_relative_scale_tween(Vec3::ONE, 400, Some(EaseFunction::BackOut)),
                300,
            )),
        ))
        .with_children(|b| {
            // ammo UI
            b.spawn((
                Name::new("ammo_sprite"),
                SpriteBundle {
                    texture: sprites.ammo_icon.clone(),
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
        });

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
        cmd.trigger(SpawnBall {
            paddle_e,
            tween_delay_ms: 1400,
        });
    }
}

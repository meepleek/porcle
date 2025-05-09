//! Spawn the main level by triggering other observers.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_enoki::ParticleEffectHandle;
use bevy_tweening::Animator;

use crate::{
    GAME_SIZE,
    game::{
        assets::{ParticleAssets, SpriteAssets},
        tween::{delay_tween, get_relative_scale_tween},
    },
    screen::Screen,
    theme::palette::{COL_AMMO_BG, COL_AMMO_FILL, COL_AMMO_OUT, COL_GEARS},
};

use super::{
    ball::SpawnBall,
    paddle::{PADDLE_RADIUS, Paddle, SpawnPaddle},
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_level)
        .add_systems(Update, (add_ball_to_paddle,));
}

pub const CORE_RADIUS: f32 = 90.0;
pub const AMMO_FILL_RADIUS: f32 = 34.0;
pub const GEAR_COUNT: u8 = 8;

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
                Sprite {
                    image: sprites.gear_small.clone_weak(),
                    color: COL_GEARS,
                    ..default()
                },
                Transform::from_translation(((rot * Vec2::X) * 71.).extend(0.1))
                    .with_rotation(Quat::from_rotation_z(angle))
                    .with_scale(Vec2::ZERO.extend(1.)),
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

    let clear_mesh_id = cmd
        .spawn((
            Name::new("clear_flash"),
            Mesh2d(meshes.add(Circle::new(PADDLE_RADIUS))),
            MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::NONE))),
            Transform::from_translation(Vec3::Z * 0.001),
        ))
        .id();

    cmd.spawn((
        Name::new("core"),
        Transform::default(),
        Visibility::default(),
        Collider::circle(CORE_RADIUS),
        CollidingEntities::default(),
        RigidBody::Static,
        Core {
            gear_entities: cog_entity_ids
                .iter()
                .cloned()
                .map(|e| (e, true))
                .rev()
                .collect(),
            clear_mesh_e: clear_mesh_id,
        },
        Health(GEAR_COUNT),
        StateScoped(Screen::Game),
    ))
    .add_children(&cog_entity_ids)
    .with_children(|b| {
        b.spawn((
            Transform::from_scale(Vec2::ZERO.extend(1.)),
            Visibility::default(),
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
                Sprite {
                    image: sprites.ammo_icon.clone_weak(),
                    color: COL_AMMO_BG,
                    ..default()
                },
                Transform::from_translation(Vec3::Z * 0.3),
            ));

            b.spawn((
                Name::new("ammo_fill"),
                AmmoFill,
                MeshMaterial2d(materials.add(ColorMaterial::from_color(COL_AMMO_FILL))),
                Transform::from_translation(Vec3::Z * 0.2)
                    .with_rotation(Quat::from_rotation_z(180f32.to_radians())),
            ));

            b.spawn((
                Name::new("ammo_bg"),
                Mesh2d(meshes.add(Circle::new(AMMO_FILL_RADIUS + 2.))),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(COL_AMMO_OUT))),
                Transform::from_translation(Vec3::Z * 0.1),
            ));
        });

        //particles
        b.spawn((
            particles.circle_particle_spawner(),
            ParticleEffectHandle(particles.core.clone_weak()),
        ));
    });

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

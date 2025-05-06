use std::time::Duration;

use avian2d::prelude::*;
use bevy::{prelude::*, render::mesh::AnnulusMeshBuilder};
use bevy_enoki::ParticleEffectHandle;
use bevy_tweening::Animator;

use super::level::{AmmoFill, AmmoUi, Core, Health, RotateWithPaddle};
use crate::{
    ext::TransExt,
    game::{
        assets::{ParticleAssets, SpriteAssets},
        movement::AccumulatedRotation,
        tween::{delay_tween, get_relative_scale_tween},
    },
    screen::Screen,
    theme::palette::{
        COL_AMMO_BG, COL_AMMO_FILL, COL_AMMO_OUT, COL_GEARS, COL_PADDLE, COL_PADDLE_CAPTURE,
        COL_PADDLE_CAPTURED, COL_PADDLE_REFLECT, COL_PADDLE_TRACKS,
    },
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_paddle);
}

pub const CORE_RADIUS: f32 = 90.0;
pub const AMMO_FILL_RADIUS: f32 = 34.0;
pub const GEAR_COUNT: u8 = 8;
pub const PADDLE_RADIUS: f32 = 200.0;
pub const PADDLE_HEIGHT: f32 = 80.0;
pub const PADDLE_COLL_HEIGHT: f32 = PADDLE_HEIGHT + 20.;

#[derive(Event, Debug)]
pub struct SpawnPaddle;

// todo: use relationships?
#[derive(Component, Debug)]
pub struct Paddle {
    pub sprite_e: Entity,
    pub barrel_e: Entity,
    pub reflect_e: Entity,
}

#[derive(Component, Debug)]
pub enum PaddleMode {
    Reflect,
    Capture,
    Captured {
        shoot_rotation: Rot2,
        ball_e: Entity,
    },
}

impl PaddleMode {
    pub fn color(&self) -> Color {
        match self {
            PaddleMode::Reflect => COL_PADDLE_REFLECT,
            PaddleMode::Capture => COL_PADDLE_CAPTURE,
            PaddleMode::Captured { .. } => COL_PADDLE_CAPTURED,
        }
    }
}

#[derive(Component, Debug)]
pub struct PaddleRotation {
    pub cw_start: f32,
    pub ccw_start: f32,
    pub timer: Timer,
    pub prev_rot: f32,
    pub paddle_e: Entity,
}

impl PaddleRotation {
    fn new(paddle_e: Entity) -> Self {
        Self {
            cw_start: 0.,
            ccw_start: 0.,
            timer: Timer::new(Duration::from_millis(50), TimerMode::Once),
            prev_rot: 0.,
            paddle_e,
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

#[derive(Component, Debug)]
pub struct PaddleAmmo {
    ammo: usize,
    capacity: usize,
}

impl PaddleAmmo {
    pub fn ammo(&self) -> usize {
        self.ammo
    }

    pub fn offset(&mut self, delta: isize) {
        self.ammo = ((self.ammo as isize + delta) as usize).clamp(0, self.capacity);
    }

    pub fn factor(&self) -> f32 {
        self.ammo as f32 / self.capacity as f32
    }
}

// todo: simplify spawning
fn spawn_paddle(
    _trigger: Trigger<SpawnPaddle>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    sprites: Res<SpriteAssets>,
    particles: Res<ParticleAssets>,
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
    .add_child(clear_mesh_id)
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

        // rails/paddle radius
        for (i, offset) in [-10., 15.].into_iter().enumerate() {
            let annulus_builder =
                AnnulusMeshBuilder::new(PADDLE_RADIUS + offset, PADDLE_RADIUS + offset + 10., 128);
            annulus_builder.build();
            b.spawn((
                Name::new("rail"),
                Mesh2d(meshes.add(annulus_builder.build())),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(COL_PADDLE_TRACKS))),
                Transform::zero_scale_2d(),
                Animator::new(delay_tween(
                    get_relative_scale_tween(Vec3::ONE, 600, Some(EaseFunction::BackOut)),
                    950 + i as u64 * 150,
                )),
                StateScoped(Screen::Game),
            ));
        }

        let barrel_e = b
            .spawn((Transform::default(), Visibility::default()))
            .with_children(|b| {
                b.spawn((
                    Name::new("barrel"),
                    Sprite {
                        image: sprites.paddle_barrel.clone_weak(),
                        color: COL_PADDLE,
                        ..default()
                    },
                    Transform::from_xyz(0., 55., 0.),
                ));
            })
            .id();

        let reflect_e = b
            .spawn((
                Name::new("reflect"),
                Sprite {
                    image: sprites.paddle_reflect.clone_weak(),
                    color: COL_PADDLE_REFLECT,
                    ..default()
                },
                Transform::from_xyz(0., -17.5, 0.5),
            ))
            .id();

        let sprite_e = b
            .spawn((Transform::default(), Visibility::default()))
            .with_children(|b| {
                b.spawn((
                    Name::new("base_sprite"),
                    Sprite {
                        image: sprites.paddle_base.clone_weak(),
                        color: COL_PADDLE,
                        ..default()
                    },
                    Transform::from_xyz(7., 0., 0.)
                        .with_rotation(Quat::from_rotation_z(-90f32.to_radians()))
                        .with_scale(Vec2::ZERO.extend(1.)),
                    Animator::new(delay_tween(
                        get_relative_scale_tween(Vec3::ONE, 500, Some(EaseFunction::BackOut)),
                        1200,
                    )),
                ))
                .add_child(barrel_e)
                .with_children(|b| {
                    for sign in [1., -1.] {
                        b.spawn((
                            Name::new("wheel"),
                            Sprite {
                                image: sprites.paddle_wheel.clone_weak(),
                                color: COL_PADDLE,
                                ..default()
                            },
                            Transform::from_xyz(98. * sign, -16., 0.),
                            RotateWithPaddle {
                                invert: true,
                                offset: Rot2::default(),
                                multiplier: 10.,
                            },
                        ));
                    }
                })
                .add_child(reflect_e);
            })
            .id();

        let paddle_e = b
            .spawn((
                Name::new("paddle"),
                Transform::from_xyz(PADDLE_RADIUS, 0.0, 1.0),
                Visibility::default(),
                // todo: fix - wrong position
                Collider::capsule(23.0, PADDLE_COLL_HEIGHT),
                Paddle {
                    sprite_e,
                    barrel_e,
                    reflect_e,
                },
                PaddleMode::Reflect,
                PaddleAmmo {
                    capacity: 55,
                    ammo: 0,
                },
            ))
            .add_child(sprite_e)
            .id();

        b.spawn((
            Name::new("paddle_rotation"),
            Transform::default(),
            Visibility::default(),
            PaddleRotation::new(paddle_e),
            AccumulatedRotation::default(),
            StateScoped(Screen::Game),
        ))
        .add_child(paddle_e);
    });
}

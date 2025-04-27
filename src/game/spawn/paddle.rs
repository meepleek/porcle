use std::time::Duration;

use avian2d::prelude::*;
use bevy::{
    prelude::*,
    render::mesh::AnnulusMeshBuilder,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_tweening::{Animator, EaseFunction};

use crate::{
    ext::TransExt,
    game::{
        assets::SpriteAssets,
        movement::AccumulatedRotation,
        tween::{delay_tween, get_relative_scale_tween},
    },
    screen::Screen,
    ui::palette::{
        COL_PADDLE, COL_PADDLE_CAPTURE, COL_PADDLE_CAPTURED, COL_PADDLE_REFLECT, COL_PADDLE_TRACKS,
    },
};

use super::level::RotateWithPaddle;

pub(super) fn plugin(app: &mut App) {
    app.add_observer(spawn_paddle);
}

pub const PADDLE_RADIUS: f32 = 350.0;
pub const PADDLE_HEIGHT: f32 = 120.0;
pub const PADDLE_COLL_HEIGHT: f32 = PADDLE_HEIGHT + 20.;

#[derive(Event, Debug)]
pub struct SpawnPaddle;

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

fn spawn_paddle(
    _trigger: Trigger<SpawnPaddle>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    sprites: Res<SpriteAssets>,
) {
    // rails/paddle radius
    for (i, offset) in [-10., 15.].into_iter().enumerate() {
        let annulus_builder =
            AnnulusMeshBuilder::new(PADDLE_RADIUS + offset, PADDLE_RADIUS + offset + 10., 128);
        annulus_builder.build();
        cmd.spawn((
            Name::new("rail"),
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(annulus_builder.build())),
                material: materials.add(ColorMaterial::from_color(COL_PADDLE_TRACKS)),
                transform: Transform::zero_scale_2d(),
                ..default()
            },
            Animator::new(delay_tween(
                get_relative_scale_tween(Vec3::ONE, 600, Some(EaseFunction::BackOut)),
                950 + i as u64 * 150,
            )),
            StateScoped(Screen::Game),
        ));
    }

    let barrel_e = cmd
        .spawn(SpatialBundle::default())
        .with_children(|b| {
            b.spawn((
                Name::new("barrel"),
                SpriteBundle {
                    texture: sprites.paddle_barrel.clone(),
                    sprite: Sprite {
                        color: COL_PADDLE,
                        ..default()
                    },
                    transform: Transform::from_xyz(0., 55., 0.),
                    ..default()
                },
            ));
        })
        .id();

    let reflect_e = cmd
        .spawn((
            Name::new("reflect"),
            SpriteBundle {
                texture: sprites.paddle_reflect.clone(),
                sprite: Sprite {
                    color: COL_PADDLE_REFLECT,
                    ..default()
                },
                transform: Transform::from_xyz(0., -17.5, 0.5),
                ..default()
            },
        ))
        .id();

    let sprite_e = cmd
        .spawn(SpatialBundle::default())
        .with_children(|b| {
            b.spawn((
                Name::new("base_sprite"),
                SpriteBundle {
                    texture: sprites.paddle_base.clone(),
                    sprite: Sprite {
                        color: COL_PADDLE,
                        ..default()
                    },
                    transform: Transform::from_xyz(7., 0., 0.)
                        .with_rotation(Quat::from_rotation_z(-90f32.to_radians()))
                        .with_scale(Vec2::ZERO.extend(1.)),
                    ..default()
                },
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
                        SpriteBundle {
                            texture: sprites.paddle_wheel.clone(),
                            sprite: Sprite {
                                color: COL_PADDLE,
                                ..default()
                            },
                            transform: Transform::from_xyz(98. * sign, -16., 0.),
                            ..default()
                        },
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

    let paddle_e = cmd
        .spawn((
            Name::new("paddle"),
            SpatialBundle::from_transform(Transform::from_xyz(PADDLE_RADIUS, 0.0, 1.0)),
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

    cmd.spawn((
        Name::new("paddle_rotation"),
        SpatialBundle::default(),
        PaddleRotation::new(paddle_e),
        AccumulatedRotation::default(),
        StateScoped(Screen::Game),
    ))
    .add_child(paddle_e);
}

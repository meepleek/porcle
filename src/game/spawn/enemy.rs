use std::time::Duration;

use avian2d::prelude::*;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    time::common_conditions::on_timer,
};
use rand::prelude::*;

use crate::{
    ext::Vec2Ext,
    game::movement::{HomingTarget, MovementBundle},
    screen::Screen,
};

use super::level::Health;

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_enemy);
    app.add_systems(
        Update,
        spawner.run_if(in_state(Screen::Game).and_then(on_timer(Duration::from_millis(1500)))),
    );
}

#[derive(Event, Debug)]
pub struct SpawnEnemy {
    pub kind: EnemyKind,
    pub position: Vec2,
}

#[derive(Component, Debug, Clone)]
pub struct Enemy {
    pub mesh_e: Entity,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub enum EnemyKind {
    Crawler,
}

fn spawner(mut cmd: Commands) {
    let mut rng = thread_rng();
    cmd.trigger(SpawnEnemy {
        kind: EnemyKind::Crawler,
        position: Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize() * 720.,
    });
}

fn spawn_enemy(
    trigger: Trigger<SpawnEnemy>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ev = trigger.event();
    match ev.kind {
        EnemyKind::Crawler => {
            let size = 30.;
            let a = Vec2::Y * size;
            let b = Vec2::new(-size, -size);
            let c = Vec2::new(size, -size);

            let color = bevy::color::palettes::tailwind::PURPLE_400;
            let mesh_e = cmd
                .spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Triangle2d::new(a, b, c))),
                    material: materials.add(ColorMaterial::from_color(color)),
                    ..default()
                })
                .id();

            cmd.spawn((
                Name::new("Crawler"),
                SpatialBundle::from_transform(
                    Transform::from_translation(ev.position.extend(0.1))
                        .with_rotation(ev.position.to_quat()),
                ),
                Collider::triangle(a, b, c),
                MovementBundle::new(-ev.position.normalize_or_zero(), 30.),
                HomingTarget,
                Enemy {
                    mesh_e,
                    color: color.into(),
                },
                Health(3),
                StateScoped(Screen::Game),
            ))
            .add_child(mesh_e);
        }
    }
}

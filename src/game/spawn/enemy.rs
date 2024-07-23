use avian2d::prelude::*;
use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};

use crate::{ext::Vec2Ext, screen::Screen};

pub(super) fn plugin(app: &mut App) {
    app.observe(spawn_enemy);
}

#[derive(Event, Debug)]
pub struct SpawnEnemy {
    pub enemy: Enemy,
    pub position: Vec2,
}

#[derive(Component, Debug, Clone)]
pub enum Enemy {
    Crawler,
}

fn spawn_enemy(
    trigger: Trigger<SpawnEnemy>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let ev = trigger.event();
    match ev.enemy {
        Enemy::Crawler => {
            let size = 30.;
            let a = Vec2::Y * size;
            let b = Vec2::new(-size, -size);
            let c = Vec2::new(size, -size);
            cmd.spawn((
                MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Triangle2d::new(a, b, c))),
                    material: materials.add(ColorMaterial::from_color(
                        bevy::color::palettes::tailwind::RED_400,
                    )),
                    transform: Transform::from_translation(ev.position.extend(0.1))
                        .with_rotation(ev.position.to_quat()),
                    ..default()
                },
                RigidBody::Kinematic,
                Collider::triangle(a, b, c),
                LinearVelocity(-ev.position.normalize_or_zero() * 50.),
                ev.enemy.clone(),
                StateScoped(Screen::Game),
            ));
        }
    }
}

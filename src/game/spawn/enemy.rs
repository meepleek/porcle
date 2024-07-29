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
    ui::palette::COL_ENEMY,
    GAME_SIZE,
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
    pub speed: f32,
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
    let spawn_dist = (2.0 * (GAME_SIZE / 2.0).powi(2)).sqrt() + 100.;
    cmd.trigger(SpawnEnemy {
        kind: EnemyKind::Crawler,
        position: (Rot2::degrees(rng.gen_range(-360.0..360.0)) * Vec2::X).normalize() * spawn_dist,
        speed: rng.gen_range(25.0..40.0),
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

            let mesh_e = cmd
                .spawn(MaterialMesh2dBundle {
                    mesh: Mesh2dHandle(meshes.add(Triangle2d::new(a, b, c))),
                    material: materials.add(ColorMaterial::from_color(COL_ENEMY)),
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
                MovementBundle::new(-ev.position.normalize_or_zero(), ev.speed),
                HomingTarget,
                Enemy {
                    mesh_e,
                    color: COL_ENEMY,
                },
                Health(3),
                StateScoped(Screen::Game),
            ))
            .add_child(mesh_e);
        }
    }
}

//! Spawn the main level by triggering other observers.

use bevy::prelude::*;
use bevy_ecs_ldtk::{LdtkWorldBundle, LevelSelection};

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(LevelSelection::index(0))
        .observe(spawn_level);
}

#[derive(Event, Debug)]
pub struct SpawnLevel(pub usize);

fn spawn_level(
    trigger: Trigger<SpawnLevel>,
    mut commands: Commands,
    ass: Res<AssetServer>,
    mut lvl: ResMut<LevelSelection>,
) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: ass.load("levels.ldtk"),
        ..Default::default()
    });
    *lvl = LevelSelection::index(trigger.event().0);
}

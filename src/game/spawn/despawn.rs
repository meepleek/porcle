use bevy::prelude::*;

use crate::GAME_SIZE;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, despawn_out_of_bounds);
}

#[derive(Component, Event, Debug)]
pub struct DespawnOutOfBounds;

fn despawn_out_of_bounds(
    despawn_q: Query<(Entity, &GlobalTransform), With<DespawnOutOfBounds>>,
    mut cmd: Commands,
) -> Result {
    let treshold = GAME_SIZE + 150.;
    for (e, t) in &despawn_q {
        if t.translation().truncate().max_element() > treshold {
            cmd.get_entity(e)?.despawn();
        }
    }
    Ok(())
}

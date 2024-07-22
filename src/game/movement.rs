use bevy::{prelude::*, utils::HashMap};
use bevy_ecs_ldtk::prelude::*;

use crate::AppSet;

use super::spawn::player::{NextPartIid, PrevPartIid, SnakeHead};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(Update, process_input.in_set(AppSet::ProcessInput));

    // Apply movement based on controls.
    app.add_systems(
        Update,
        (translate_grid_coords_entities,)
            .chain()
            .in_set(AppSet::Update),
    );
}

#[derive(Component)]
pub enum PartOrientation {
    Horizontal,
    Vertical,
    TopRight,
    BottomLeft,
    BottomRight,
    TopLeft,
}

fn process_input(
    mut heads: Query<(&mut GridCoords, &EntityIid), With<SnakeHead>>,
    input: Res<ButtonInput<KeyCode>>,
    mut parts: Query<
        (
            Entity,
            &mut GridCoords,
            &EntityIid,
            &NextPartIid,
            Option<&PrevPartIid>,
        ),
        Without<SnakeHead>,
    >,
    mut cmd: Commands,
) {
    let movement_direction =
        if input.just_pressed(KeyCode::KeyW) || input.just_pressed(KeyCode::ArrowUp) {
            GridCoords::new(0, 1)
        } else if input.just_pressed(KeyCode::KeyA) || input.just_pressed(KeyCode::ArrowLeft) {
            GridCoords::new(-1, 0)
        } else if input.just_pressed(KeyCode::KeyS) || input.just_pressed(KeyCode::ArrowDown) {
            GridCoords::new(0, -1)
        } else if input.just_pressed(KeyCode::KeyD) || input.just_pressed(KeyCode::ArrowRight) {
            GridCoords::new(1, 0)
        } else {
            return;
        };

    let mut part_coords: HashMap<_, _> = parts
        .iter()
        .map(|(_, coords, iid, _, _)| (iid.clone(), coords.clone()))
        .collect();

    for (mut head_coords, iid) in heads.iter_mut() {
        part_coords.insert(iid.clone(), head_coords.clone());
        let destination = *head_coords + movement_direction;
        *head_coords = destination;
    }

    for (e, mut coords, _, next_part_iid, prev_part_iid) in parts.iter_mut() {
        let next_coords = part_coords
            .get(&next_part_iid.0)
            .expect("Next coords are always set");
        *coords = *next_coords;
        let prev_coords = prev_part_iid.and_then(|prev_part_iid| part_coords.get(&prev_part_iid.0));

        match prev_coords {
            Some(prev_coords) => {
                // body
                let orientation = if prev_coords.x == next_coords.x {
                    PartOrientation::Vertical
                } else if prev_coords.y == next_coords.y {
                    PartOrientation::Horizontal
                } else if (next_coords.y > coords.y && prev_coords.x > coords.x)
                    || (prev_coords.y > coords.y && next_coords.x > coords.x)
                {
                    PartOrientation::TopRight
                } else if (next_coords.y < coords.y && prev_coords.x > coords.x)
                    || (prev_coords.y < coords.y && next_coords.x > coords.x)
                {
                    PartOrientation::BottomRight
                } else if (next_coords.y < coords.y && prev_coords.x < coords.x)
                    || (prev_coords.y < coords.y && next_coords.x < coords.x)
                {
                    PartOrientation::BottomLeft
                } else {
                    PartOrientation::TopLeft
                };
                cmd.entity(e).insert(orientation);
            }
            None => {
                // tail
            }
        }
    }
}

// todo: tween
// todo: move to new file
const GRID_SIZE: i32 = 32;

fn translate_grid_coords_entities(
    mut grid_coords_entities: Query<(&mut Transform, &GridCoords), Changed<GridCoords>>,
) {
    for (mut transform, grid_coords) in grid_coords_entities.iter_mut() {
        transform.translation =
            bevy_ecs_ldtk::utils::grid_coords_to_translation(*grid_coords, IVec2::splat(GRID_SIZE))
                .extend(transform.translation.z);
    }
}

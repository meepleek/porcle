use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::AppSet;

use super::spawn::player::SnakeHead;

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.register_type::<MovementController>();
    app.add_systems(Update, process_input.in_set(AppSet::ProcessInput));

    // Apply movement based on controls.
    app.add_systems(
        Update,
        (translate_grid_coords_entities,)
            .chain()
            .in_set(AppSet::Update),
    );
}

#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct MovementController(pub IVec2);

fn process_input(
    mut players: Query<&mut GridCoords, With<SnakeHead>>,
    input: Res<ButtonInput<KeyCode>>,
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

    for mut player_grid_coords in players.iter_mut() {
        let destination = *player_grid_coords + movement_direction;
        *player_grid_coords = destination;
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

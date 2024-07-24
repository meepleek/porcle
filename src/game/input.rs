use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CursorCoords>()
        .add_systems(Update, my_cursor_system);
}

#[derive(Resource, Default)]
pub struct CursorCoords(pub Vec2);

fn my_cursor_system(
    mut mycoords: ResMut<CursorCoords>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<IsDefaultUiCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();

    // check the cursor is inside the window and get its position
    // then convert into world coordinates
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        mycoords.0 = world_position;
    }
}

use bevy::input::gamepad::GamepadEvent;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use leafwing_input_manager::plugin::InputManagerSystem;
use leafwing_input_manager::prelude::*;

use crate::math::asymptotic_smoothing_with_delta_time;
use crate::AppSet;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<CursorCoords>()
        .add_systems(
            Update,
            ((update_cursor_coords, update_aim_direction).chain(),)
                .in_set(AppSet::ProcessInput)
                .after(InputManagerSystem::ManualControl),
        )
        .add_plugins(InputManagerPlugin::<PlayerAction>::default())
        .init_resource::<AimDirection>()
        .init_resource::<ActionState<PlayerAction>>()
        .insert_resource(PlayerAction::input_map())
        .init_state::<ActiveInput>()
        .add_systems(
            Update,
            activate_gamepad.run_if(in_state(ActiveInput::MouseKeyboard)),
        )
        .add_systems(Update, activate_mkb.run_if(in_state(ActiveInput::Gamepad)));
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum PlayerAction {
    Shoot,
    TogglePaddleMode,
    AimGamepad,
    Quit,
    Restart,
}

#[derive(Resource, Debug, Default, Reflect)]
pub struct AimDirection(pub Vec2);

impl PlayerAction {
    fn input_map() -> InputMap<Self> {
        let mut input_map = InputMap::default();

        // Gamepad
        let deadzone_radius = 0.15;
        let deadzone = DeadZoneShape::Ellipse {
            radius_x: deadzone_radius,
            radius_y: deadzone_radius,
        };
        input_map.insert(
            Self::AimGamepad,
            DualAxis::left_stick().with_deadzone(deadzone),
        );
        input_map.insert(
            Self::AimGamepad,
            DualAxis::right_stick().with_deadzone(deadzone),
        );
        input_map.insert(Self::Shoot, GamepadButtonType::RightTrigger);
        input_map.insert(Self::Shoot, GamepadButtonType::RightTrigger2);
        input_map.insert(Self::Shoot, GamepadButtonType::South);
        input_map.insert(Self::TogglePaddleMode, GamepadButtonType::LeftTrigger);
        input_map.insert(Self::TogglePaddleMode, GamepadButtonType::LeftTrigger2);
        input_map.insert(Self::TogglePaddleMode, GamepadButtonType::West);
        input_map.insert(Self::Restart, GamepadButtonType::Start);
        input_map.insert(Self::Quit, GamepadButtonType::Select);

        // KB & Mouse
        input_map.insert(Self::Shoot, MouseButton::Left);
        input_map.insert(Self::TogglePaddleMode, MouseButton::Right);
        input_map.insert(Self::Quit, KeyCode::Escape);
        input_map.insert(Self::Restart, KeyCode::KeyR);

        input_map
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum ActiveInput {
    #[default]
    MouseKeyboard,
    Gamepad,
}

pub type PlayerInput<'w> = Res<'w, ActionState<PlayerAction>>;

#[derive(Resource, Default)]
struct CursorCoords(pub Vec2);

fn update_aim_direction(
    mut aim_dir: ResMut<AimDirection>,
    input_state: Res<State<ActiveInput>>,
    cursor: Res<CursorCoords>,
    input: PlayerInput,
    time: Res<Time>,
) {
    aim_dir.0 = match input_state.get() {
        ActiveInput::MouseKeyboard => {
            let deadzone_radius = 70.0;
            let dist = cursor.0.length();
            if dist >= deadzone_radius {
                cursor.0.normalize_or(aim_dir.0)
            } else {
                asymptotic_smoothing_with_delta_time(
                    aim_dir.0,
                    cursor.0,
                    (dist / deadzone_radius).powi(3),
                    time.delta_seconds(),
                )
                // aim_dir.0
            }
        }
        ActiveInput::Gamepad => input
            .clamped_axis_pair(&PlayerAction::AimGamepad)
            .map_or(aim_dir.0, |dir| dir.xy().normalize_or(aim_dir.0)),
    }
}

fn update_cursor_coords(
    mut coords: ResMut<CursorCoords>,
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
        coords.0 = world_position;
    }
}

fn activate_gamepad(
    mut next_state: ResMut<NextState<ActiveInput>>,
    mut gamepad_evr: EventReader<GamepadEvent>,
) {
    for ev in gamepad_evr.read() {
        match ev {
            GamepadEvent::Button(_) | GamepadEvent::Axis(_) => {
                debug!("Switching to gamepad input");
                next_state.set(ActiveInput::Gamepad);
                return;
            }
            _ => (),
        }
    }
}

/// Switch to mouse and keyboard input when any keyboard button is pressed
fn activate_mkb(
    mut next_state: ResMut<NextState<ActiveInput>>,
    mut kb_evr: EventReader<KeyboardInput>,
    mut mouse_btn_evr: EventReader<MouseButtonInput>,
    mut cursor_evr: EventReader<CursorMoved>,
) {
    if !mouse_btn_evr.is_empty() || !cursor_evr.is_empty() || !kb_evr.is_empty() {
        debug!("Switching to mouse and keyboard input");
        next_state.set(ActiveInput::MouseKeyboard);
        mouse_btn_evr.clear();
        cursor_evr.clear();
        kb_evr.clear();
    }
}

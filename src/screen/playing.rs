//! The screen state for the main game loop.

use bevy::{
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use leafwing_input_manager::common_conditions::action_just_pressed;

use super::{NextTransitionedState, Screen};
use crate::game::{
    // assets::SoundtrackKey,
    audio::soundtrack::PlayMusic,
    input::PlayerAction,
    score::Score,
    spawn::level::SpawnLevel,
};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Game), enter_playing)
        .add_systems(OnExit(Screen::Game), exit_playing)
        .add_systems(
            OnEnter(Screen::RestartGame),
            super::enter_screen(Screen::Game),
        )
        .add_systems(
            Update,
            (
                super::enter_screen(Screen::Title)
                    .run_if(in_state(Screen::Game).and(action_just_pressed(PlayerAction::Quit))),
                super::enter_screen(Screen::RestartGame)
                    .run_if(in_state(Screen::Game).and(action_just_pressed(PlayerAction::Restart))),
            ),
        );
}

fn enter_playing(
    mut cmd: Commands,
    mut window_q: Query<&mut Window, With<PrimaryWindow>>,
    mut score: ResMut<Score>,
) {
    cmd.trigger(SpawnLevel);
    // commands.trigger(PlaySoundtrack::Key(SoundtrackKey::Gameplay));
    // reset score
    score.0 = 0;

    if cfg!(not(any(target_family = "wasm", target_os = "macos"))) {
        let mut win = window_q.single_mut().expect("window exists");
        win.cursor_options.grab_mode = CursorGrabMode::Confined;
    }
}

fn exit_playing(mut commands: Commands, mut window_q: Query<&mut Window, With<PrimaryWindow>>) {
    // We could use [`StateScoped`] on the sound playing entites instead.
    commands.trigger(PlayMusic::Disable);

    if cfg!(not(any(target_family = "wasm", target_os = "macos"))) {
        let mut win: Mut<'_, Window> = window_q.single_mut().expect("window exists");
        win.cursor_options.grab_mode = CursorGrabMode::None;
    }
}

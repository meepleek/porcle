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
        .add_systems(OnEnter(Screen::RestartGame), enter_restart)
        .add_systems(
            Update,
            (
                return_to_title_screen.run_if(
                    in_state(Screen::Game).and_then(action_just_pressed(PlayerAction::Quit)),
                ),
                restart_game.run_if(
                    in_state(Screen::Game).and_then(action_just_pressed(PlayerAction::Restart)),
                ),
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
    let mut win = window_q.single_mut();
    // reset score
    score.0 = 0;

    #[cfg(not(target_family = "wasm"))]
    {
        win.cursor.grab_mode = CursorGrabMode::Confined;
    }
}

fn exit_playing(mut commands: Commands, mut window_q: Query<&mut Window, With<PrimaryWindow>>) {
    // We could use [`StateScoped`] on the sound playing entites instead.
    commands.trigger(PlayMusic::Disable);
    let mut win = window_q.single_mut();

    #[cfg(not(target_family = "wasm"))]
    {
        win.cursor.grab_mode = CursorGrabMode::None;
    }
}

fn return_to_title_screen(mut next_screen: ResMut<NextTransitionedState>) {
    next_screen.set(Screen::Title);
}

fn restart_game(mut next_screen: ResMut<NextTransitionedState>) {
    next_screen.set(Screen::RestartGame);
}

fn enter_restart(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Game);
}

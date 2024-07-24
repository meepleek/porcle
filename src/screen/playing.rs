//! The screen state for the main game loop.

use bevy::{
    input::common_conditions::input_just_pressed,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

use super::Screen;
use crate::game::{
    // assets::SoundtrackKey,
    audio::soundtrack::PlaySoundtrack,
    spawn::{
        level::{Core, SpawnLevel},
        paddle::PaddleAmmo,
    },
};
use crate::ui::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Game), enter_playing)
        .add_systems(OnExit(Screen::Game), exit_playing)
        .add_systems(OnEnter(Screen::RestartGame), enter_restart)
        .add_systems(
            Update,
            (
                update_state_text.run_if(in_state(Screen::Game)),
                return_to_title_screen
                    .run_if(in_state(Screen::Game).and_then(input_just_pressed(KeyCode::Escape))),
                restart_game
                    .run_if(in_state(Screen::Game).and_then(input_just_pressed(KeyCode::KeyR))),
            ),
        );
}

fn enter_playing(mut commands: Commands, mut window_q: Query<&mut Window, With<PrimaryWindow>>) {
    commands.trigger(SpawnLevel);
    // commands.trigger(PlaySoundtrack::Key(SoundtrackKey::Gameplay));
    let mut win = window_q.single_mut();
    win.cursor.grab_mode = CursorGrabMode::Confined;

    commands
        .ui_root()
        .insert(Style {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::FlexStart,
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(StateScoped(Screen::Game))
        .with_children(|children| {
            children.header("");
        });
}

fn exit_playing(mut commands: Commands, mut window_q: Query<&mut Window, With<PrimaryWindow>>) {
    // We could use [`StateScoped`] on the sound playing entites instead.
    commands.trigger(PlaySoundtrack::Disable);
    let mut win = window_q.single_mut();
    win.cursor.grab_mode = CursorGrabMode::None;
}

fn return_to_title_screen(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}

fn restart_game(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::RestartGame);
}

fn enter_restart(mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Game);
}

fn update_state_text(
    mut text_q: Query<&mut Text>,
    ammo_q: Query<&PaddleAmmo>,
    core_q: Query<&Core>,
) {
    if let (Ok(core), Ok(ammo)) = (core_q.get_single(), ammo_q.get_single()) {
        let mut text = text_q.single_mut();
        text.sections[0].value = format!("Health: {} - Ammo: {}", core.health, ammo.0).to_string();
    }
}

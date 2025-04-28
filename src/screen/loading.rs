//! A loading screen during which game assets are loaded.
//! This reduces stuttering, especially for audio on WASM.

use bevy::prelude::*;

use super::{NextTransitionedState, Screen};
use crate::{
    game::{
        assets::{MusicAssets, ParticleAssets},
        audio::soundtrack::PlayMusic,
    },
    ui::prelude::*,
};

// todo: use transition
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), enter_loading)
        .add_systems(OnEnter(Screen::Loaded), on_loaded);
}

fn enter_loading(mut commands: Commands) {
    commands
        .ui_root()
        .insert(StateScoped(Screen::Loaded))
        .with_children(|children| {
            children.label("Loading...");
        });
}

fn on_loaded(
    mut next_screen: ResMut<NextTransitionedState>,
    mut cmd: Commands,
    particles: Res<ParticleAssets>,
    music: Res<MusicAssets>,
) {
    next_screen.set(Screen::Title);
    // bg particles
    cmd.spawn((particles.circle_particle_spawner(), particles.bg.clone()));
    cmd.trigger(PlayMusic::Track(music.track_1.clone()));
}

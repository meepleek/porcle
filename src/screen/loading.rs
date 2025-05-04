//! A loading screen during which game assets are loaded.
//! This reduces stuttering, especially for audio on WASM.

use bevy::prelude::*;
use bevy_enoki::ParticleEffectHandle;

use super::{NextTransitionedState, Screen};
use crate::{
    game::{
        assets::{MusicAssets, ParticleAssets},
        audio::soundtrack::PlayMusic,
    },
    theme::widget,
};

// todo: use transition
pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(Screen::Loading), enter_loading)
        .add_systems(OnEnter(Screen::Loaded), on_loaded);
}

fn enter_loading(mut commands: Commands) {
    commands.spawn((
        StateScoped(Screen::Loaded),
        widget::ui_root("loading"),
        children![widget::label("Loading...")],
    ));
}

fn on_loaded(
    mut next_screen: ResMut<NextTransitionedState>,
    mut cmd: Commands,
    particles: Res<ParticleAssets>,
    music: Res<MusicAssets>,
) {
    next_screen.set(Screen::Title);
    // bg particles
    cmd.spawn((
        particles.circle_particle_spawner(),
        ParticleEffectHandle(particles.bg.clone_weak()),
    ));
    cmd.trigger(PlayMusic::Track(music.track_1.clone()));
}

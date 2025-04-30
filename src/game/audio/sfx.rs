use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(play_sfx);
}

fn play_sfx(trigger: Trigger<PlaySfx>, mut commands: Commands) {
    commands.spawn((
        AudioPlayer(trigger.event().0.clone()),
        PlaybackSettings {
            mode: PlaybackMode::Despawn,
            volume: Volume::new(0.175),
            ..default()
        },
    ));
}

#[derive(Event)]
pub struct PlaySfx(pub Handle<AudioSource>);

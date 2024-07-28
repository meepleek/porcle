use bevy::{audio::PlaybackMode, prelude::*};

pub(super) fn plugin(app: &mut App) {
    app.observe(play_sfx);
}

fn play_sfx(trigger: Trigger<PlaySfx>, mut commands: Commands) {
    commands.spawn(AudioSourceBundle {
        source: trigger.event().0.clone(),
        settings: PlaybackSettings {
            mode: PlaybackMode::Despawn,
            ..default()
        },
    });
}

#[derive(Event)]
pub struct PlaySfx(pub Handle<AudioSource>);

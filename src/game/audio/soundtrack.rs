use bevy::{
    audio::{PlaybackMode, Volume},
    prelude::*,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Music>();
    app.add_observer(play_soundtrack);
}

fn play_soundtrack(
    trigger: Trigger<PlayMusic>,
    mut commands: Commands,
    soundtrack_query: Query<Entity, With<Music>>,
) {
    for entity in &soundtrack_query {
        commands.entity(entity).despawn_recursive();
    }

    let handle = match trigger.event() {
        PlayMusic::Track(key) => key.clone(),
        PlayMusic::Disable => return,
    };
    commands.spawn((
        AudioPlayer(handle),
        PlaybackSettings {
            mode: PlaybackMode::Loop,
            volume: Volume::Linear(0.275),
            ..default()
        },
        Music,
    ));
}

#[derive(Event)]
pub enum PlayMusic {
    #[allow(dead_code)]
    Track(Handle<AudioSource>),
    Disable,
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct Music;

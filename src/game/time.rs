use bevy::prelude::*;
use std::marker::PhantomData;

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(Update, process_cooldown::<()>);
}

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Cooldown<T: Send + Sync + 'static> {
    timer: Timer,
    _phantom: PhantomData<T>,
}

impl<T: Send + Sync> Cooldown<T> {
    pub fn new(duration_s: f32) -> Self {
        Self {
            timer: Timer::from_seconds(duration_s, TimerMode::Once),
            _phantom: default(),
        }
    }
}

pub fn process_cooldown<T: Send + Sync>(
    mut cmd: Commands,
    mut cooldown_q: Query<(Entity, &mut Cooldown<T>)>,
    time: Res<Time>,
) {
    for (e, mut cooldown) in cooldown_q.iter_mut() {
        cooldown.timer.tick(time.delta());

        if cooldown.timer.just_finished() {
            cmd.entity(e).remove::<Cooldown<T>>();
        }
    }
}

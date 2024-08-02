use bevy::prelude::*;
use std::time::Duration;

pub trait SendDelayedEventExt<T> {
    fn send_delayed_event(&mut self, event: T, delay_ms: u64);
}

impl<'w, 's, T: Event> SendDelayedEventExt<T> for Commands<'w, 's> {
    fn send_delayed_event(&mut self, event: T, delay_ms: u64) {
        self.spawn(DelayedEvent::new(event, delay_ms));
    }
}

#[derive(Component)]
pub struct DelayedEvent<T: Event> {
    pub event: Option<T>,
    timer: Timer,
}

impl<T: Event> DelayedEvent<T> {
    pub fn new(event: T, delay_ms: u64) -> Self {
        Self {
            event: Some(event),
            timer: Timer::new(Duration::from_millis(delay_ms), TimerMode::Once),
        }
    }
}

pub fn send_delayed_event<T: Event>(
    mut ev_w: EventWriter<T>,
    mut delayed_q: Query<(Entity, &mut DelayedEvent<T>)>,
    time: Res<Time>,
    mut cmd: Commands,
) {
    for (e, mut delayed) in &mut delayed_q {
        delayed.timer.tick(time.delta());
        if delayed.timer.just_finished() {
            if let Some(ev) = delayed.event.take() {
                ev_w.send(ev);
            }
            cmd.entity(e).remove::<DelayedEvent<T>>();
        }
    }
}

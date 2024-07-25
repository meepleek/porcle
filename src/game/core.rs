use avian2d::prelude::*;
use bevy::prelude::*;
use bevy_trauma_shake::Shakes;

use crate::screen::Screen;

use super::spawn::{
    enemy::Enemy,
    level::{Core, Health},
};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(Update, handle_collisions);
}

fn handle_collisions(
    mut core_q: Query<(&mut Health, &CollidingEntities), With<Core>>,
    enemy_q: Query<(), With<Enemy>>,
    mut cmd: Commands,
    mut next: ResMut<NextState<Screen>>,
    mut shake: Shakes,
) {
    for (mut hp, coll) in &mut core_q {
        for coll_e in coll.iter() {
            if enemy_q.contains(*coll_e) {
                cmd.entity(*coll_e).despawn_recursive();
                hp.0 = hp.0.saturating_sub(1);
                info!("ouch!");

                if hp.0 == 0 {
                    next.set(Screen::GameOver);
                    shake.add_trauma(0.55);
                } else {
                    shake.add_trauma(0.3);
                }
            }
        }
    }
}

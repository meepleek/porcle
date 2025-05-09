//! Game mechanics and content.

use bevy::prelude::*;

pub mod assets;
pub mod audio;
pub mod ball;
mod core;
mod gun;
pub mod input;
mod movement;
pub mod paddle;
pub mod score;
pub mod spawn;
pub mod time;
pub mod tween;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        audio::plugin,
        assets::plugin,
        movement::plugin,
        spawn::plugin,
        input::plugin,
        gun::plugin,
        core::plugin,
        time::plugin,
        tween::plugin,
        ball::plugin,
        paddle::plugin,
        score::plugin,
    ));
}

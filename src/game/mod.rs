//! Game mechanics and content.

use bevy::prelude::*;

pub mod assets;
pub mod audio;
mod core;
mod gun;
pub mod input;
mod movement;
pub mod spawn;
pub mod time;

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
    ));
}

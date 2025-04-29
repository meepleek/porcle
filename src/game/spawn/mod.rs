//! Handles spawning of entities. Here, we are using
//! [observers](https://docs.rs/bevy/latest/bevy/ecs/prelude/struct.add_observerr.html)
//! for this, but you could also use `Events<E>` or `Commands`.

use bevy::prelude::*;

pub mod ball;
pub mod despawn;
pub mod enemy;
pub mod level;
pub mod paddle;
pub mod projectile;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        level::plugin,
        paddle::plugin,
        ball::plugin,
        enemy::plugin,
        projectile::plugin,
        despawn::plugin,
    ));
}

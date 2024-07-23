use avian2d::prelude::*;
use bevy::prelude::*;

use crate::AppSet;

use super::{
    input::CursorCoords,
    spawn::{ball::Ball, paddle::PaddleRotation},
};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(
        Update,
        (
            process_input.in_set(AppSet::ProcessInput),
            rotate_paddle,
            reflect_ball,
        ),
    );
}

fn process_input(_input: Res<ButtonInput<KeyCode>>, mut _cmd: Commands) {}

fn rotate_paddle(
    mut rot_q: Query<&mut Transform, With<PaddleRotation>>,
    cursor: Res<CursorCoords>,
) {
    for mut t in rot_q.iter_mut() {
        if let Ok(dir) = Dir2::new(cursor.0) {
            t.rotation = Quat::from_rotation_z(dir.to_angle());
        }
    }
}

fn reflect_ball(
    mut coll_q: Query<(Entity, &CollidingEntities, &mut LinearVelocity), With<Ball>>,
    collisions: Res<Collisions>,
) {
    for (e, colliding, mut vel) in &mut coll_q {
        if !colliding.is_empty() {
            if let Some(coll) = collisions.get(e, *colliding.0.iter().next().unwrap()) {
                if let Some(contact) = coll.manifolds.first() {
                    vel.0 = contact.normal1 * -250.;
                    info!(
                        ?coll,
                        "{:?} is colliding with the following entities: {:?}", e, colliding
                    );
                }
            }
        }
    }
}

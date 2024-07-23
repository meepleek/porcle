use bevy::prelude::*;

use crate::{ext::Vec2Ext, game::spawn::projectile::SpawnProjectile};

use super::spawn::paddle::PaddleAmmo;

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_systems(Update, fire_gun);
}

fn fire_gun(
    mut ammo_q: Query<(&mut PaddleAmmo, &GlobalTransform)>,
    input: Res<ButtonInput<MouseButton>>,
    mut cmd: Commands,
) {
    if input.just_pressed(MouseButton::Left) {
        for (mut ammo, t) in &mut ammo_q {
            // todo: cooldown
            if ammo.0 > 0 {
                let dir = Dir2::new(t.right().truncate()).unwrap();
                let rot = t.up().truncate().to_quat();
                cmd.trigger(SpawnProjectile {
                    dir,
                    transform: Transform::from_translation(
                        t.translation() + (rot * (-Vec3::Y * 70.0)),
                    )
                    .with_rotation(rot),
                });
                ammo.0 -= 1;
            }
        }
    }
}

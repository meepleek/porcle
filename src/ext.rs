use bevy::prelude::*;

pub trait Vec2Ext {
    fn to_quat(self) -> Quat;
}

impl Vec2Ext for Vec2 {
    fn to_quat(self) -> Quat {
        match Dir2::new(self) {
            Ok(dir) => Quat::from_rotation_z(dir.to_angle()),
            Err(_) => Quat::IDENTITY,
        }
    }
}

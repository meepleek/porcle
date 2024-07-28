use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Score>();
}

#[derive(Resource, Debug, Default, Reflect)]
pub struct Score(pub usize);

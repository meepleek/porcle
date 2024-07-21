use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_ldtk_entity::<PickupGrowBundle>("PickupGrow");
}

#[derive(Default, Bundle, LdtkEntity)]
struct PickupGrowBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
}

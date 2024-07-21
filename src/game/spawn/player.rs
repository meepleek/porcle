use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_ldtk_entity::<SnakeHeadBundle>("SnakeHead")
        .register_ldtk_entity::<SnakeBodyBundle>("SnakeBody")
        .register_ldtk_entity::<SnakeTailBundle>("SnakeTail");
}

#[derive(Default, Component)]
pub struct SnakeHead;

#[derive(Default, Bundle, LdtkEntity)]
struct SnakeHeadBundle {
    head: SnakeHead,
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Default, Component)]
pub struct NextPartIid(pub EntityIid);

#[derive(Default, Bundle, LdtkEntity)]
struct SnakeBodyBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    #[with(get_next_iid_field)]
    next_iid: NextPartIid,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Default, Bundle, LdtkEntity)]
struct SnakeTailBundle {
    #[sprite_sheet_bundle]
    sprite_sheet_bundle: LdtkSpriteSheetBundle,
    #[with(get_next_iid_field)]
    next_iid: NextPartIid,
    #[grid_coords]
    grid_coords: GridCoords,
}

pub fn get_next_iid_field(entity_instance: &EntityInstance) -> NextPartIid {
    let iid = entity_instance
        .get_entity_ref_field("next")
        .expect("expected entity to have next entity ref field");
    NextPartIid(EntityIid::new(iid.entity_iid.clone()))
}

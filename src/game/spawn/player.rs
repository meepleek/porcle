use bevy::{prelude::*, utils::hashbrown::HashMap};
use bevy_ecs_ldtk::prelude::*;

// todo: instead of reconstructing the snake parts just collect the snake, keep all the parts in 1 component to make processing/updating much easier
pub(super) fn plugin(app: &mut App) {
    app.register_ldtk_entity::<SnakeHeadBundle>("SnakeHead")
        .register_ldtk_entity::<SnakeBodyBundle>("SnakeBody")
        .register_ldtk_entity::<SnakeTailBundle>("SnakeTail")
        .add_systems(Update, process_part);
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

#[derive(Default, Component)]
pub struct PrevPartIid(pub EntityIid);

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

fn process_part(
    mut cmd: Commands,
    iid_q: Query<(Entity, &EntityIid, &NextPartIid), Added<NextPartIid>>,
) {
    let iids: HashMap<_, _> = iid_q
        .iter()
        .map(|(_, iid, next_iid)| (next_iid.0.clone(), iid.clone()))
        .collect();

    for (e, iid, _) in iid_q.iter() {
        if let Some(prev_iid) = iids.get(iid) {
            cmd.entity(e).insert(PrevPartIid(prev_iid.clone()));
        }
    }
}

use bevy::prelude::*;

use crate::game::{
    assets::{SfxAssets, assets_exist},
    audio::sfx::PlaySfx,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<InteractionPalette>();
    app.add_systems(
        Update,
        (apply_interaction_palette, trigger_interaction_sfx).run_if(assets_exist),
    );
}

/// Palette for widget interactions.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub struct InteractionPalette {
    pub none: Color,
    pub hovered: Color,
    pub pressed: Color,
}

fn apply_interaction_palette(
    mut palette_query: Query<
        (&Interaction, &InteractionPalette, &mut BackgroundColor),
        Changed<Interaction>,
    >,
) {
    for (interaction, palette, mut background) in &mut palette_query {
        *background = match interaction {
            Interaction::None => palette.none,
            Interaction::Hovered => palette.hovered,
            Interaction::Pressed => palette.pressed,
        }
        .into();
    }
}

fn trigger_interaction_sfx(
    mut interactions: Query<&Interaction, Changed<Interaction>>,
    mut commands: Commands,
    sfx: Res<SfxAssets>,
) {
    for interaction in &mut interactions {
        match interaction {
            Interaction::Hovered => commands.trigger(PlaySfx(sfx.button_hover.clone())),
            Interaction::Pressed => commands.trigger(PlaySfx(sfx.button_click.clone())),
            _ => (),
        }
    }
}

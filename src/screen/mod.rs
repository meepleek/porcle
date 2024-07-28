//! The game's main screen states and transitions between them.

mod credits;
mod game_over;
mod loading;
mod playing;
mod splash;
mod title;

use bevy::{color::palettes::tailwind, prelude::*};

use crate::game::{
    assets::{HandleMap, SpriteKey},
    tween::{tween_factor, TweenFactor},
};

use self::splash::SPLASH_BACKGROUND_COLOR;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>()
        .enable_state_scoped_entities::<Screen>()
        .add_plugins((
            splash::plugin,
            loading::plugin,
            title::plugin,
            credits::plugin,
            playing::plugin,
            game_over::plugin,
        ))
        .add_systems(Startup, setup_transition_overlay)
        .add_systems(
            Update,
            (
                start_transition_anim.run_if(state_changed::<Screen>),
                scale_transition_circles,
                fade_transition_circles,
                tween_factor::<TransitionCircle>,
                tween_factor::<FinalTransitionCircle>,
            ),
        );
}

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
pub enum Screen {
    #[default]
    Splash,
    Loading,
    Title,
    Credits,
    Game,
    RestartGame,
    GameOver,
}

#[derive(Component, Debug, Default)]
pub struct TransitionCircle;

#[derive(Component, Debug, Default)]
pub struct FinalTransitionCircle;

#[derive(Resource)]
struct Transition {
    circle_entity_ids: Vec<Entity>,
}

fn setup_transition_overlay(mut cmd: Commands, sprites: ResMut<HandleMap<SpriteKey>>) {
    let colors = [
        tailwind::CYAN_200.into(),
        tailwind::CYAN_400.into(),
        tailwind::CYAN_600.into(),
        SPLASH_BACKGROUND_COLOR,
    ];

    let circle_entity_ids: Vec<_> = colors
        .iter()
        .enumerate()
        .map(|(i, color)| {
            let mut builder = cmd.spawn((
                Name::new("transition_circle"),
                TransitionCircle,
                ImageBundle {
                    image: UiImage {
                        texture: sprites.get(&SpriteKey::TransitionCircle).unwrap().clone(),
                        color: *color,
                        ..default()
                    },
                    style: Style {
                        position_type: PositionType::Absolute,
                        width: Val::Vw(0.),
                        height: Val::Vw(0.),
                        ..default()
                    },
                    ..default()
                },
            ));
            if i == colors.len() - 1 {
                builder.insert(FinalTransitionCircle);
            }
            builder.id()
        })
        .collect();

    cmd.spawn((
        Name::new("Transition"),
        NodeBundle {
            z_index: ZIndex::Global(1000),
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        },
    ))
    .push_children(&circle_entity_ids);

    cmd.insert_resource(Transition { circle_entity_ids });
}

fn start_transition_anim(
    trans: Res<Transition>,
    mut cmd: Commands,
    circle_q: Query<
        (),
        Or<(
            With<TweenFactor<TransitionCircle>>,
            With<TweenFactor<FinalTransitionCircle>>,
        )>,
    >,
) {
    if !circle_q.is_empty() {
        return;
    }

    for (i, e) in trans.circle_entity_ids.iter().cloned().enumerate() {
        cmd.entity(e).try_insert(
            TweenFactor::<TransitionCircle>::new(1000, bevy_tweening::EaseFunction::SineInOut)
                .with_delay((i * 250) as u64),
        );
    }
}

fn scale_transition_circles(
    mut circle_q: Query<
        (
            Entity,
            &TweenFactor<TransitionCircle>,
            Option<&FinalTransitionCircle>,
        ),
        Changed<TweenFactor<TransitionCircle>>,
    >,
    mut style_q: Query<&mut Style>,
    reset_circle_q: Query<Entity, (With<TransitionCircle>, Without<FinalTransitionCircle>)>,
    mut cmd: Commands,
) {
    for (e, factor, final_circle) in &mut circle_q {
        let factor = factor.factor();
        if let Ok(mut style) = style_q.get_mut(e) {
            let size = Val::VMax(145.0 * factor);
            style.width = size;
            style.height = size;
        }

        if factor >= 1. {
            cmd.entity(e).remove::<TweenFactor<TransitionCircle>>();
            if final_circle.is_some() {
                cmd.entity(e)
                    .try_insert(TweenFactor::<FinalTransitionCircle>::new(
                        300,
                        bevy_tweening::EaseFunction::QuadraticIn,
                    ));

                // reset size of non-final circles
                for e in &reset_circle_q {
                    if let Ok(mut style) = style_q.get_mut(e) {
                        let size = Val::VMax(0.);
                        style.width = size;
                        style.height = size;
                    }
                }
            }
        }
    }
}

fn fade_transition_circles(
    mut final_circle_q: Query<(
        Entity,
        &mut Style,
        &mut UiImage,
        &TweenFactor<FinalTransitionCircle>,
    )>,
    mut cmd: Commands,
) {
    if let Ok((e, mut style, mut image, factor)) = final_circle_q.get_single_mut() {
        let factor = factor.factor();
        image.color.set_alpha(1.0 - factor);
        if factor >= 1. {
            // reset transition back
            image.color.set_alpha(1.0);
            let size = Val::VMax(0.);
            style.width = size;
            style.height = size;
            cmd.entity(e).remove::<TweenFactor<FinalTransitionCircle>>();
        }
    }
}

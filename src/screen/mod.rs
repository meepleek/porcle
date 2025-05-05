//! The game's main screen states and transitions between them.

mod credits;
mod game_over;
mod loading;
mod playing;
mod splash;
mod title;
mod tutorial;

use bevy::{prelude::*, window::WindowResized};

use crate::{
    game::{
        assets::{SpriteAssets, assets_exist},
        tween::{TweenFactor, tween_factor},
    },
    theme::palette::{COL_BG, COL_LETTERBOX, COL_TRANSITION_1, COL_TRANSITION_2, COL_TRANSITION_3},
};

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Screen>()
        .enable_state_scoped_entities::<Screen>()
        .init_resource::<NextTransitionedState>()
        .add_plugins((
            splash::plugin,
            loading::plugin,
            title::plugin,
            credits::plugin,
            playing::plugin,
            game_over::plugin,
            tutorial::plugin,
        ))
        .add_systems(OnExit(Screen::Loading), setup_transition_overlay)
        .add_systems(Startup, setup_letterbox)
        .add_systems(
            Update,
            (
                resize_letterbox,
                start_transition_anim.run_if(
                    assets_exist
                        .and(resource_exists::<Transition>)
                        .and(resource_changed::<NextTransitionedState>),
                ),
                transition_out,
                transition_in,
                tween_factor::<TransitionCircle>,
                tween_factor::<FinalTransitionCircle>,
            ),
        );
}

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Copy, Clone, Default)]
pub enum Screen {
    #[default]
    Splash,
    Loading,
    Loaded,
    Title,
    Credits,
    Tutorial,
    Game,
    RestartGame,
    GameOver,
    Exit,
}

pub fn in_game_state(current_state: Option<Res<State<Screen>>>) -> bool {
    match current_state {
        Some(current_state) => *current_state == Screen::Game,
        None => false,
    }
}

pub fn enter_screen_on_pointer_click(
    screen: Screen,
) -> impl FnMut(Trigger<Pointer<Click>>, ResMut<NextTransitionedState>) {
    let mut enter = enter_screen(screen);
    move |_, next_screen| {
        enter(next_screen);
    }
}

pub fn enter_screen(screen: Screen) -> impl FnMut(ResMut<NextTransitionedState>) {
    move |mut next_screen| {
        next_screen.set(screen);
    }
}

#[derive(Component, Debug, Default)]
pub struct TransitionCircle;

#[derive(Component, Debug, Default)]
pub struct FinalTransitionCircle;

#[derive(Resource)]
struct Transition {
    circle_entity_ids: Vec<Entity>,
}

#[derive(Resource, Default)]
pub struct NextTransitionedState(Option<Screen>);
impl NextTransitionedState {
    pub fn set(&mut self, next: Screen) {
        self.0 = Some(next);
    }
}

fn setup_transition_overlay(mut cmd: Commands, sprites: ResMut<SpriteAssets>) {
    let colors = [COL_TRANSITION_1, COL_TRANSITION_2, COL_TRANSITION_3, COL_BG];

    let circle_entity_ids: Vec<_> = colors
        .iter()
        .enumerate()
        .map(|(i, color)| {
            let mut builder = cmd.spawn((
                Name::new("transition_circle"),
                TransitionCircle,
                ImageNode::new(sprites.transition_circle.clone_weak()).with_color(*color),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Vw(0.),
                    height: Val::Vw(0.),
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
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        GlobalZIndex(1000),
        Pickable::IGNORE,
    ))
    .add_children(&circle_entity_ids);

    cmd.insert_resource(Transition { circle_entity_ids });
}

#[derive(Component)]
enum LetterboxAxis {
    Vertical,
    Horizontal,
}

fn setup_letterbox(mut cmd: Commands) {
    cmd.spawn((
        Name::new("letterbox"),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        GlobalZIndex(1500),
        Pickable::IGNORE,
    ))
    .with_children(|b| {
        let bg_color: BackgroundColor = COL_LETTERBOX.into();
        b.spawn((
            Name::new("letterbox_left"),
            bg_color,
            Node {
                position_type: PositionType::Absolute,
                top: Val::ZERO,
                left: Val::ZERO,
                width: Val::ZERO,
                height: Val::Vh(100.),
                ..default()
            },
            LetterboxAxis::Vertical,
        ));
        b.spawn((
            Name::new("letterbox_right"),
            bg_color,
            Node {
                position_type: PositionType::Absolute,
                top: Val::ZERO,
                right: Val::ZERO,
                width: Val::ZERO,
                height: Val::Vh(100.),
                ..default()
            },
            LetterboxAxis::Vertical,
        ));
        b.spawn((
            Name::new("letterbox_top"),
            bg_color,
            Node {
                position_type: PositionType::Absolute,
                top: Val::ZERO,
                left: Val::ZERO,
                width: Val::Vw(100.),
                height: Val::ZERO,
                ..default()
            },
            LetterboxAxis::Horizontal,
        ));
        b.spawn((
            Name::new("letterbox_bottom"),
            bg_color,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::ZERO,
                left: Val::ZERO,
                width: Val::Vw(100.),
                height: Val::ZERO,
                ..default()
            },
            LetterboxAxis::Horizontal,
        ));
    });
}

fn resize_letterbox(
    mut letterbox_q: Query<(&LetterboxAxis, &mut Node)>,
    mut resize_evr: EventReader<WindowResized>,
) {
    if let Some(ev) = resize_evr.read().next() {
        for (axis, mut style) in &mut letterbox_q {
            match axis {
                LetterboxAxis::Vertical => {
                    style.width = Val::Px((ev.width - ev.height).max(0.) / 2.);
                }
                LetterboxAxis::Horizontal => {
                    style.height = Val::Px((ev.height - ev.width).max(0.) / 2.);
                }
            }
        }
    }
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
    next_transitioned: Res<NextTransitionedState>,
) {
    if !circle_q.is_empty() || next_transitioned.0.is_none() {
        return;
    }

    for (i, e) in trans.circle_entity_ids.iter().cloned().enumerate() {
        cmd.entity(e).try_insert(
            TweenFactor::<TransitionCircle>::new(800, EaseFunction::SineInOut)
                .with_delay((i * 150) as u64),
        );
    }
}

fn transition_out(
    mut circle_q: Query<
        (
            Entity,
            &TweenFactor<TransitionCircle>,
            Option<&FinalTransitionCircle>,
        ),
        Changed<TweenFactor<TransitionCircle>>,
    >,
    mut style_q: Query<&mut Node>,
    reset_circle_q: Query<Entity, (With<TransitionCircle>, Without<FinalTransitionCircle>)>,
    mut cmd: Commands,
    next_transitioned: Res<NextTransitionedState>,
    mut next_state: ResMut<NextState<Screen>>,
) {
    for (e, factor, final_circle) in &mut circle_q {
        let factor = factor.factor();
        if let Ok(mut style) = style_q.get_mut(e) {
            let size = Val::VMax(145.0 * factor);
            style.width = size;
            style.height = size;
        }

        if factor >= 1. {
            if let Some(new_state) = &next_transitioned.0 {
                next_state.set(*new_state);
            }
            cmd.entity(e).remove::<TweenFactor<TransitionCircle>>();
            if final_circle.is_some() {
                cmd.entity(e)
                    .try_insert(TweenFactor::<FinalTransitionCircle>::new(
                        200,
                        EaseFunction::QuadraticIn,
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

fn transition_in(
    mut final_circle_q: Query<(
        Entity,
        &mut Node,
        &mut ImageNode,
        &TweenFactor<FinalTransitionCircle>,
    )>,
    mut cmd: Commands,
) {
    if let Ok((e, mut style, mut image, factor)) = final_circle_q.single_mut() {
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

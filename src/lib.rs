#[cfg(feature = "dev")]
mod dev_tools;
mod ext;
mod game;
mod math;
mod screen;
mod ui;

use bevy::{
    asset::AssetMetaCheck,
    audio::{AudioPlugin, Volume},
    core_pipeline::{
        bloom::{Bloom, BloomCompositeMode},
        tonemapping::Tonemapping,
    },
    prelude::*,
    render::camera::ScalingMode,
};
use bevy_trauma_shake::ShakeSettings;

pub struct AppPlugin;

pub const GAME_SIZE: f32 = 1600.;
pub const MIN_WINDOW_SIZE: f32 = 720.;
pub const BLOOM_BASE: f32 = 0.15;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Order new `AppStep` variants by adding them here:
        app.configure_sets(
            Update,
            (AppSet::TickTimers, AppSet::ProcessInput, AppSet::Update).chain(),
        );

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);

        // Add Bevy plugins.
        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    // Wasm builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Porcle".to_string(),
                        canvas: Some("#bevy".to_string()),
                        fit_canvas_to_parent: true,
                        resolution: Vec2::splat(1024.).into(),
                        resize_constraints: WindowResizeConstraints {
                            min_width: MIN_WINDOW_SIZE,
                            min_height: MIN_WINDOW_SIZE,
                            ..default()
                        },
                        prevent_default_event_handling: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                })
                // pixelart
                .set(ImagePlugin::default_linear())
                .set(AudioPlugin {
                    global_volume: GlobalVolume {
                        volume: Volume::new(0.3),
                    },
                    ..default()
                }),
        );

        // Add project plugins.
        app.add_plugins((game::plugin, screen::plugin, ui::plugin));

        // Add external plugins
        app.add_plugins((
            avian2d::PhysicsPlugins::default(),
            bevy_trauma_shake::TraumaPlugin,
            bevy_enoki::EnokiPlugin,
        ));

        // Enable dev tools for dev builds.
        #[cfg(feature = "dev")]
        app.add_plugins(dev_tools::plugin);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum AppSet {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    ProcessInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

fn spawn_camera(mut commands: Commands) {
    let mut ortho_projection = OrthographicProjection::default_2d();
    ortho_projection.scaling_mode = ScalingMode::AutoMin {
        min_width: GAME_SIZE,
        min_height: GAME_SIZE,
    };

    commands.spawn((
        Name::new("Camera"),
        Camera2d,
        Camera {
            hdr: true,
            ..default()
        },
        ortho_projection,
        Tonemapping::TonyMcMapface,
        // Render all UI to this camera.
        // Not strictly necessary since we only use one camera,
        // but if we don't use this component, our UI will disappear as soon
        // as we add another camera. This includes indirect ways of adding cameras like using
        // [ui node outlines](https://bevyengine.org/news/bevy-0-14/#ui-node-outline-gizmos)
        // for debugging. So it's good to have this here for future-proofing.
        IsDefaultUiCamera,
        bevy_trauma_shake::Shake::default(),
        ShakeSettings::default(),
        Bloom {
            intensity: BLOOM_BASE,
            high_pass_frequency: 0.5,
            low_frequency_boost: 0.3,
            low_frequency_boost_curvature: 0.7,
            composite_mode: BloomCompositeMode::Additive,
            ..default()
        },
    ));
}

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_enoki::prelude::*;

use crate::screen::Screen;

pub(super) fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(Screen::Loading)
            .continue_to_state(Screen::Loaded)
            .load_collection::<SpriteAssets>()
            .load_collection::<SfxAssets>()
            .load_collection::<MusicAssets>(),
    );
    app.add_systems(Startup, setup_particles);
}

pub fn assets_exist(
    sprites: Option<Res<SpriteAssets>>,
    sfx: Option<Res<SfxAssets>>,
    music: Option<Res<MusicAssets>>,
    particles: Option<Res<ParticleAssets>>,
) -> bool {
    sprites.is_some() && sfx.is_some() && music.is_some() && particles.is_some()
}

#[derive(AssetCollection, Resource)]
pub struct SpriteAssets {
    #[asset(path = "images/transition_circle.png")]
    pub transition_circle: Handle<Image>,
    #[asset(path = "images/gear_small.png")]
    pub gear_small: Handle<Image>,
    #[asset(path = "images/ammo_icon.png")]
    pub ammo_icon: Handle<Image>,
    #[asset(path = "images/bullet.png")]
    pub bullet: Handle<Image>,
    #[asset(path = "images/paddle_base.png")]
    pub paddle_base: Handle<Image>,
    #[asset(path = "images/paddle_reflect.png")]
    pub paddle_reflect: Handle<Image>,
    #[asset(path = "images/paddle_wheel.png")]
    pub paddle_wheel: Handle<Image>,
    #[asset(path = "images/paddle_barrel.png")]
    pub paddle_barrel: Handle<Image>,
    #[asset(path = "images/ball.png")]
    pub ball: Handle<Image>,
    #[asset(path = "images/enemy_creepinek.png")]
    pub enemy_creepinek: Handle<Image>,
    #[asset(path = "images/enemy_creepy_shield.png")]
    pub enemy_creepy_shield: Handle<Image>,
    #[asset(path = "images/enemy_big_boi.png")]
    pub enemy_big_boi: Handle<Image>,
    #[asset(path = "images/enemy_bang.png")]
    pub enemy_bang: Handle<Image>,
    #[asset(path = "images/enemy_bang_barrel.png")]
    pub enemy_bang_barrel: Handle<Image>,
    #[asset(path = "images/enemy_projectile.png")]
    pub enemy_projectile: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct SfxAssets {
    #[asset(path = "audio/sfx/button_hover.ogg")]
    pub button_hover: Handle<AudioSource>,
    #[asset(path = "audio/sfx/button_press.ogg")]
    pub button_click: Handle<AudioSource>,
}

#[derive(AssetCollection, Resource)]
pub struct MusicAssets {
    #[asset(path = "audio/soundtracks/track_1.ogg")]
    pub track_1: Handle<AudioSource>,
}

// todo: use asset_loader for particles too
// #[derive(AssetCollection, Resource)]
// pub struct ParticleAssets {
//     #[asset(path = "particles/circle.png")]
//     pub circle_mat: Handle<SpriteParticle2dMaterial>,
//     #[asset(path = "particles/gun.particle.ron")]
//     pub gun: ParticleEffectHandle,
//     #[asset(path = "particles/enemy.particle.ron")]
//     pub enemy: ParticleEffectHandle,
//     #[asset(path = "particles/reflection.particle.ron")]
//     pub reflection: ParticleEffectHandle,
//     #[asset(path = "particles/core.particle.ron")]
//     pub core: ParticleEffectHandle,
// }

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ParticleAssets {
    pub circle_mat: Handle<SpriteParticle2dMaterial>,
    pub gun: Handle<Particle2dEffect>,
    pub enemy: Handle<Particle2dEffect>,
    pub reflection: Handle<Particle2dEffect>,
    pub core: Handle<Particle2dEffect>,
    pub core_clear: Handle<Particle2dEffect>,
    pub bg: Handle<Particle2dEffect>,
    pub ball: Handle<Particle2dEffect>,
}

impl ParticleAssets {
    pub fn square_particle_spawner(&self) -> ParticleSpawner<ColorParticle2dMaterial> {
        ParticleSpawner::default()
    }

    pub fn circle_particle_spawner(&self) -> ParticleSpawner<SpriteParticle2dMaterial> {
        ParticleSpawner(self.circle_mat.clone_weak())
    }
}

fn setup_particles(
    ass: Res<AssetServer>,
    mut materials: ResMut<Assets<SpriteParticle2dMaterial>>,
    mut cmd: Commands,
) {
    cmd.insert_resource(ParticleAssets {
        circle_mat: materials.add(
            // hframes and vframes define how the sprite sheet is divided for animations,
            // if you just want to bind a single texture, leave both at 1.
            SpriteParticle2dMaterial::new(ass.load("particles/circle.png"), 1, 1),
        ),
        gun: ass.load("particles/gun.particle.ron"),
        enemy: ass.load("particles/enemy.particle.ron"),
        reflection: ass.load("particles/reflection.particle.ron"),
        core: ass.load("particles/core.particle.ron"),
        core_clear: ass.load("particles/core_clear.particle.ron"),
        bg: ass.load("particles/bg.particle.ron"),
        ball: ass.load("particles/ball.particle.ron"),
    });
}

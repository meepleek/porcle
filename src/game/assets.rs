use bevy::{prelude::*, utils::HashMap};
use bevy_enoki::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<HandleMap<SfxKey>>();
    app.init_resource::<HandleMap<SfxKey>>();

    app.register_type::<HandleMap<SoundtrackKey>>();
    app.init_resource::<HandleMap<SoundtrackKey>>();
    app.add_systems(Startup, setup_particles);
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum SfxKey {
    ButtonHover,
    ButtonPress,
    Step1,
    Step2,
    Step3,
    Step4,
}

impl AssetKey for SfxKey {
    type Asset = AudioSource;
}

impl FromWorld for HandleMap<SfxKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (
                SfxKey::ButtonHover,
                asset_server.load("audio/sfx/button_hover.ogg"),
            ),
            (
                SfxKey::ButtonPress,
                asset_server.load("audio/sfx/button_press.ogg"),
            ),
            (SfxKey::Step1, asset_server.load("audio/sfx/step1.ogg")),
            (SfxKey::Step2, asset_server.load("audio/sfx/step2.ogg")),
            (SfxKey::Step3, asset_server.load("audio/sfx/step3.ogg")),
            (SfxKey::Step4, asset_server.load("audio/sfx/step4.ogg")),
        ]
        .into()
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Reflect)]
pub enum SoundtrackKey {
    Credits,
    Gameplay,
}

impl AssetKey for SoundtrackKey {
    type Asset = AudioSource;
}

impl FromWorld for HandleMap<SoundtrackKey> {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        [
            (
                SoundtrackKey::Credits,
                asset_server.load("audio/soundtracks/Monkeys Spinning Monkeys.ogg"),
            ),
            (
                SoundtrackKey::Gameplay,
                asset_server.load("audio/soundtracks/Fluffing A Duck.ogg"),
            ),
        ]
        .into()
    }
}

pub trait AssetKey: Sized {
    type Asset: Asset;
}

#[derive(Resource, Reflect, Deref, DerefMut)]
#[reflect(Resource)]
pub struct HandleMap<K: AssetKey>(HashMap<K, Handle<K::Asset>>);

impl<K: AssetKey, T> From<T> for HandleMap<K>
where
    T: Into<HashMap<K, Handle<K::Asset>>>,
{
    fn from(value: T) -> Self {
        Self(value.into())
    }
}

impl<K: AssetKey> HandleMap<K> {
    pub fn all_loaded(&self, asset_server: &AssetServer) -> bool {
        self.values()
            .all(|x| asset_server.is_loaded_with_dependencies(x))
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ParticleAssets {
    pub circle_mat: Handle<SpriteParticle2dMaterial>,
    pub gun: Handle<Particle2dEffect>,
    pub enemy: Handle<Particle2dEffect>,
    pub reflection: Handle<Particle2dEffect>,
}

impl ParticleAssets {
    pub fn particle_spawner(
        &self,
        effect: Handle<Particle2dEffect>,
        transform: Transform,
    ) -> ParticleSpawnerBundle<SpriteParticle2dMaterial> {
        ParticleSpawnerBundle {
            effect,
            material: self.circle_mat.clone(),
            transform,
            ..default()
        }
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
    });
}

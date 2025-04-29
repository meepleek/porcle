use std::ops::Range;

use bevy::prelude::*;

use crate::{GAME_SIZE, ext::QuatExt};

use super::time::{Cooldown, process_cooldown};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<MoveDirection>()
        .register_type::<Damping>()
        .register_type::<Speed>()
        .register_type::<Velocity>()
        .add_systems(First, insert_velocity)
        .add_systems(
            Update,
            (
                process_cooldown::<MovementPaused>,
                (
                    apply_damping,
                    compute_velocity.after(apply_damping),
                    apply_impulse.after(compute_velocity),
                    home.after(apply_impulse),
                )
                    .before(ApplyVelocitySet),
                apply_velocity.in_set(ApplyVelocitySet),
                (accumulate_angle, follow).after(ApplyVelocitySet),
            ),
        );
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ApplyVelocitySet;

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct Velocity(Vec2);

impl Velocity {
    pub fn velocity(&self) -> Vec2 {
        self.0
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
#[require(Impulse)]
pub struct MoveDirection(pub Vec2);

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct Damping(pub f32);

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct Speed(pub f32);

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct SpeedMultiplier(pub f32);

#[derive(Component, Debug, Default, Deref, DerefMut, Reflect)]
pub struct Impulse(pub Vec2);

#[derive(Component, Debug, Reflect)]
pub struct MovementPaused;

impl MovementPaused {
    pub fn cooldown(duration_s: f32) -> Cooldown<MovementPaused> {
        Cooldown::new(duration_s)
    }
}

impl Speed {
    pub fn speed_factor(&self, min: f32, max: f32) -> f32 {
        speed_factor(self.0, min, max)
    }
}

pub fn speed_factor(speed: f32, min: f32, max: f32) -> f32 {
    ((speed - min) / max).clamp(0., 1.)
}

#[derive(Component, Debug)]
pub struct Homing {
    pub max_distance: f32,
    pub max_factor: f32,
    pub factor_decay: f32,
    pub max_angle: f32,
    pub speed_mult: Option<Range<f32>>,
}

#[derive(Component, Debug)]
pub struct HomingTarget;

#[derive(Component, Debug)]
pub struct Follow {
    pub offset: Vec2,
    pub entity: Entity,
}

#[derive(Component, Debug, Default)]
pub struct AccumulatedRotation {
    prev: Option<Rot2>,
    pub rotation: f32,
}

fn insert_velocity(add_q: Query<Entity, Added<MoveDirection>>, mut cmd: Commands) {
    for e in &add_q {
        cmd.entity(e).try_insert(Velocity::default());
    }
}

fn compute_velocity(
    mut move_q: Query<
        (
            &MoveDirection,
            &Speed,
            Option<&SpeedMultiplier>,
            &mut Velocity,
        ),
        (Without<MovementPaused>, Without<Cooldown<MovementPaused>>),
    >,
    time: Res<Time>,
) {
    for (dir, speed, speed_mult, mut vel) in &mut move_q {
        vel.0 = dir.0 * speed.0 * speed_mult.map_or(1.0, |m| m.0) * time.delta_secs();
    }
}

fn apply_impulse(
    mut impulse_q: Query<
        (&mut Impulse, &mut Velocity),
        (Without<MovementPaused>, Without<Cooldown<MovementPaused>>),
    >,
    time: Res<Time>,
) {
    for (mut impulse, mut vel) in &mut impulse_q {
        let mult_delta = time.delta_secs() * 6.5;
        vel.0 += impulse.0 * mult_delta;
        // fixme: this is incorrect, but that can wait after the jam
        impulse.0 *= 1. - mult_delta;
    }
}

fn apply_velocity(
    mut move_q: Query<
        (&mut Transform, &Velocity),
        (Without<MovementPaused>, Without<Cooldown<MovementPaused>>),
    >,
) {
    for (mut t, vel) in &mut move_q {
        t.translation += vel.0.extend(0.);
    }
}

fn home(
    mut move_q: Query<
        (&Transform, &mut Velocity, &MoveDirection, &Homing, &Speed),
        (Without<MovementPaused>, Without<Cooldown<MovementPaused>>),
    >,
    time: Res<Time>,
    target_q: Query<&GlobalTransform, With<HomingTarget>>,
) {
    for (homing_t, mut vel, move_dir, homing, speed) in &mut move_q {
        let speed_factor = homing
            .speed_mult
            .as_ref()
            .map_or(1., |range| speed.speed_factor(range.start, range.end));

        if speed_factor <= 0. {
            continue;
        }

        let mut closest_distance = f32::MAX;
        let mut homing_target_dir = None;

        for target_t in target_q.iter() {
            // todo: need to fix this
            if target_t.translation().abs().max_element() > (GAME_SIZE / 2.0 - 50.) {
                // outside window
                continue;
            }

            let distance = homing_t.translation.distance(target_t.translation());

            if distance < closest_distance && distance <= homing.max_distance {
                let target_dir = (target_t.translation() - homing_t.translation)
                    .normalize()
                    .truncate();
                let angle = move_dir.angle_to(target_dir).to_degrees().abs();

                if angle > homing.max_angle {
                    continue;
                }

                closest_distance = distance;
                homing_target_dir = Some(target_dir);
            }
        }

        if let Some(target_dir) = homing_target_dir {
            // Exponential decay to make homing effect stronger
            let distance_factor = (1.0 - (closest_distance / homing.max_distance))
                .powf(homing.factor_decay)
                * homing.max_factor
                * speed_factor
                * time.delta_secs();
            let homing_dir = (move_dir.0 * (1.0 - distance_factor) + target_dir * distance_factor)
                .normalize_or_zero();
            let speed = vel.0.length();
            vel.0 = homing_dir * speed;

            // todo: use if rotation is ever needed
            // homing_t.rotation = homing_dir.to_quat();
        }
    }
}

fn apply_damping(
    mut damping_q: Query<
        (&mut Velocity, &Damping, Option<&mut Speed>),
        (Without<MovementPaused>, Without<Cooldown<MovementPaused>>),
    >,
    time: Res<Time>,
) {
    for (mut vel, damping, speed) in &mut damping_q {
        let mult = 1. - (damping.0 * time.delta_secs());
        vel.0 *= mult;
        if let Some(mut speed) = speed {
            speed.0 *= mult;
        }
    }
}

fn follow(mut follow_q: Query<(&mut Transform, &Follow)>, followed_q: Query<&GlobalTransform>) {
    for (mut t, follow) in &mut follow_q {
        if let Ok(followed_t) = followed_q.get(follow.entity) {
            t.translation = followed_t.translation() + follow.offset.extend(0.);
        }
    }
}

fn accumulate_angle(mut acc_q: Query<(&mut AccumulatedRotation, &Transform), Changed<Transform>>) {
    for (mut acc, t) in &mut acc_q {
        let rot = t.rotation.to_rot2();
        if let Some(prev) = acc.prev {
            acc.rotation += prev.angle_to(rot);
        }
        acc.prev = Some(rot);
    }
}

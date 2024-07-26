use avian2d::math::Vector2;
use bevy::prelude::*;

use crate::ext::QuatExt;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(First, add_velocity).add_systems(
        Update,
        (
            (
                apply_damping,
                compute_velocity.after(apply_damping),
                apply_impulse.after(compute_velocity),
            )
                .before(ApplyVelocitySet),
            (apply_velocity, apply_homing_velocity).in_set(ApplyVelocitySet),
            (accumulate_angle, follow).after(ApplyVelocitySet),
        ),
    );
}

#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct ApplyVelocitySet;

#[derive(Bundle, Default)]
pub struct MovementBundle {
    direction: MoveDirection,
    speed: Speed,
    impulse: Impulse,
}

impl MovementBundle {
    pub fn new(dir: Vec2, speed: f32) -> Self {
        Self {
            direction: MoveDirection(dir),
            speed: Speed(speed),
            impulse: Impulse(Vector2::ZERO),
        }
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct Velocity(Vec2);

impl Velocity {
    pub fn velocity(&self) -> Vec2 {
        self.0
    }
}

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct MoveDirection(pub Vec2);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct Damping(pub f32);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct Speed(pub f32);

#[derive(Component, Debug, Default, Deref, DerefMut)]
pub struct Impulse(pub Vec2);

#[derive(Component, Debug)]
pub struct MovementPaused;

impl Speed {
    pub fn speed_factor(&self, min: f32, max: f32) -> f32 {
        ((self.0 - min) / max).clamp(0., 1.)
    }
}

#[derive(Component, Debug)]
pub struct Homing {
    pub max_distance: f32,
    pub max_factor: f32,
    pub factor_decay: f32,
    pub max_angle: f32,
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

fn add_velocity(
    add_q: Query<Entity, (Added<MoveDirection>, Without<Velocity>)>,
    mut cmd: Commands,
) {
    for e in &add_q {
        cmd.entity(e).try_insert(Velocity::default());
    }
}

fn compute_velocity(
    mut move_q: Query<(&MoveDirection, &Speed, &mut Velocity), Without<MovementPaused>>,
    time: Res<Time>,
) {
    for (dir, speed, mut vel) in &mut move_q {
        vel.0 = dir.0 * speed.0 * time.delta_seconds();
    }
}

fn apply_impulse(
    mut impulse_q: Query<(&mut Impulse, &mut Velocity), Without<MovementPaused>>,
    time: Res<Time>,
) {
    for (mut impulse, mut vel) in &mut impulse_q {
        let mult_delta = time.delta_seconds() * 6.5;
        vel.0 += impulse.0 * mult_delta;
        // fixme: this is incorrect, but that can wait after the jam
        impulse.0 *= 1. - mult_delta;
    }
}

fn apply_velocity(
    mut move_q: Query<(&mut Transform, &Velocity), (Without<Homing>, Without<MovementPaused>)>,
) {
    for (mut t, vel) in &mut move_q {
        t.translation += vel.0.extend(0.);
    }
}

fn apply_homing_velocity(
    mut move_q: Query<(&mut Transform, &mut Velocity, &Homing)>,
    time: Res<Time>,
    target_q: Query<&GlobalTransform, (With<HomingTarget>, Without<MovementPaused>)>,
) {
    for (mut homing_t, mut vel, homing) in &mut move_q {
        let dir = vel.0.normalize_or_zero();
        let mut closest_distance = f32::MAX;
        let mut homing_target_dir = None;

        for target_t in target_q.iter() {
            let distance = homing_t.translation.distance(target_t.translation());

            if distance < closest_distance && distance <= homing.max_distance {
                let target_dir = (target_t.translation() - homing_t.translation)
                    .normalize()
                    .truncate();
                let angle = dir.angle_between(target_dir).to_degrees().abs();

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
                * time.delta_seconds();
            let homing_dir =
                (dir * (1.0 - distance_factor) + target_dir * distance_factor).normalize_or_zero();
            let speed = vel.0.length();
            vel.0 = homing_dir * speed;

            // todo: use if rotation is ever needed
            // homing_t.rotation = homing_dir.to_quat();
        }

        homing_t.translation += vel.0.extend(0.);
    }
}

fn apply_damping(
    mut damping_q: Query<(&mut Velocity, &Damping, Option<&mut Speed>), Without<MovementPaused>>,
    time: Res<Time>,
) {
    for (mut vel, damping, speed) in &mut damping_q {
        let mult = 1. - (damping.0 * time.delta_seconds());
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
            acc.rotation += prev.angle_between(rot);
        }
        acc.prev = Some(rot);
    }
}

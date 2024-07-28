use avian2d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*, sprite::Mesh2dHandle};
use bevy_trauma_shake::Shakes;

use crate::{
    ext::QuatExt,
    screen::{NextTransitionedState, Screen},
};

use super::{
    movement::MovementPaused,
    spawn::{
        enemy::Enemy,
        level::{AmmoFill, Core, Gear, Health, AMMO_FILL_RADIUS},
        paddle::{PaddleAmmo, PaddleRotation},
    },
    tween::{get_relative_scale_anim, get_relative_sprite_color_anim},
};

pub(super) fn plugin(app: &mut App) {
    // Record directional input as movement controls.
    app.add_event::<TakenDamage>().add_systems(
        Update,
        (
            handle_collisions,
            rotate_gears,
            disable_gears,
            update_ammo_fill,
        ),
    );
}

#[derive(Event, Default)]
pub struct TakenDamage;

fn handle_collisions(
    mut core_q: Query<(&mut Health, &CollidingEntities), With<Core>>,
    enemy_q: Query<(), With<Enemy>>,
    mut cmd: Commands,
    mut next: ResMut<NextTransitionedState>,
    mut shake: Shakes,
    mut taken_dmg_w: EventWriter<TakenDamage>,
) {
    for (mut hp, coll) in &mut core_q {
        for coll_e in coll.iter() {
            if enemy_q.contains(*coll_e) {
                cmd.entity(*coll_e).despawn_recursive();
                hp.0 = hp.0.saturating_sub(1);
                taken_dmg_w.send_default();
                debug!("ouch!");

                if hp.0 == 0 {
                    next.set(Screen::GameOver);
                    shake.add_trauma(0.6);
                } else {
                    shake.add_trauma(0.6);
                }
            }
        }
    }
}

fn rotate_gears(
    paddle_rot_q: Query<&Transform, With<PaddleRotation>>,
    mut gear_q: Query<(&mut Transform, &Gear), (Without<PaddleRotation>, Without<MovementPaused>)>,
) {
    if let Some(paddle_t) = paddle_rot_q.iter().next() {
        for (mut gear_t, gear) in &mut gear_q {
            gear_t.rotation = Quat::from_rotation_z(
                (gear.offset.as_radians() + paddle_t.rotation.z_angle_rad())
                    * (if gear.even { 1. } else { -1. }),
            );
        }
    }
}

fn update_ammo_fill(
    ammo_q: Query<&PaddleAmmo, Changed<PaddleAmmo>>,
    ammo_fill_q: Query<Entity, With<AmmoFill>>,
    mut cmd: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if let Some(ammo) = ammo_q.iter().next() {
        for e in &ammo_fill_q {
            cmd.entity(e)
                .try_insert(Mesh2dHandle(meshes.add(CircularSegment::from_turns(
                    AMMO_FILL_RADIUS,
                    // not sure why, but the segments fills at 95% already
                    ammo.factor() * 0.95,
                ))));
        }
    }
}

fn disable_gears(
    mut ev_r: EventReader<TakenDamage>,
    mut core_q: Query<&mut Core>,
    mut cmd: Commands,
) {
    if let Ok(mut core) = core_q.get_single_mut() {
        for _ in ev_r.read() {
            if let Some((e, active)) = core.gear_entity_ids.iter_mut().find(|(_, active)| *active) {
                *active = false;
                cmd.entity(*e).try_insert((
                    get_relative_scale_anim(
                        Vec2::splat(0.7).extend(1.),
                        350,
                        Some(bevy_tweening::EaseFunction::BackIn),
                    ),
                    get_relative_sprite_color_anim(tailwind::GRAY_800.into(), 350, None),
                    MovementPaused,
                ));
            }
        }
    }
}

use bevy::prelude::*;

pub const BUTTON_HOVERED_BACKGROUND: Color = Color::srgb(0.186, 0.328, 0.573);
pub const BUTTON_PRESSED_BACKGROUND: Color = Color::srgb(0.286, 0.478, 0.773);

pub const BUTTON_TEXT: Color = Color::srgb(0.925, 0.925, 0.925);
pub const LABEL_TEXT: Color = Color::srgb(0.867, 0.827, 0.412);
pub const HEADER_TEXT: Color = Color::srgb(0.867, 0.827, 0.412);

pub const NODE_BACKGROUND: Color = Color::srgb(0.286, 0.478, 0.773);

// looks pretty nice
// todo: couple contrast issues
// also should probly replace the white with something slightly darker
// https://lospec.com/palette-list/citrink
// pub const COL_BG: Color = Color::srgb(0.13, 0.08, 0.20);
// pub const COL_TRANSITION_1: Color = COL_ENEMY;
// pub const COL_TRANSITION_2: Color = COL_ENEMY_PROJECTILE;
// pub const COL_TRANSITION_3: Color = COL_GEARS_DISABLED;
// pub const COL_BG_PARTICLES: Color = COL_PADDLE;
// pub const COL_PADDLE: Color = Color::srgb(1.0, 1.0, 1.0);
// pub const COL_PADDLE_TRACKS: Color = COL_PADDLE;
// pub const COL_BULLET: Color = Color::srgb(0.99, 0.96, 0.38);
// pub const COL_PADDLE_REFLECT: Color = COL_PADDLE_CAPTURED;
// pub const COL_PADDLE_CAPTURED: Color = Color::srgb(0.70, 0.85, 0.26);
// pub const COL_PADDLE_CAPTURE: Color = Color::srgb(0.32, 0.76, 0.25);
// pub const COL_BALL: Color = COL_PADDLE_CAPTURE;
// pub const COL_BALL_FAST: Color = COL_PADDLE_CAPTURED;
// pub const COL_GEARS: Color = Color::srgb(0.15, 0.30, 0.44);
// pub const COL_GEARS_DISABLED: Color = Color::srgb(0.15, 0.14, 0.27);
// pub const COL_AMMO_BG: Color = COL_GEARS;
// pub const COL_AMMO_FILL: Color = COL_PADDLE_CAPTURE;
// pub const COL_AMMO_OUT: Color = COL_GEARS_DISABLED;
// pub const COL_ENEMY: Color = Color::srgb(0.17, 0.48, 0.53);
// pub const COL_ENEMY_FLASH: Color = COL_PADDLE;
// pub const COL_ENEMY_PROJECTILE: Color = COL_GEARS;

// not super cohesive
// https://lospec.com/palette-list/funkyfuture-8
pub const COL_BG: Color = Color::srgb(0.17, 0.06, 0.33);
pub const COL_TRANSITION_1: Color = COL_ENEMY;
pub const COL_TRANSITION_2: Color = COL_ENEMY_PROJECTILE;
pub const COL_TRANSITION_3: Color = COL_GEARS_DISABLED;
pub const COL_BG_PARTICLES: Color = COL_PADDLE;
pub const COL_PADDLE: Color = Color::srgb(1.0, 0.97, 0.97);
pub const COL_PADDLE_TRACKS: Color = COL_PADDLE;
pub const COL_BULLET: Color = Color::srgb(1.0, 0.85, 0.27);
pub const COL_PADDLE_REFLECT: Color = COL_GEARS;
pub const COL_PADDLE_CAPTURED: Color = Color::srgb(1.0, 0.51, 0.26);
pub const COL_PADDLE_CAPTURE: Color = COL_BULLET;
pub const COL_BALL: Color = COL_PADDLE_CAPTURE;
pub const COL_BALL_FAST: Color = COL_PADDLE_CAPTURED;
pub const COL_GEARS: Color = Color::srgb(0.26, 0.83, 0.84);
pub const COL_GEARS_DISABLED: Color = Color::srgb(0.2, 0.41, 0.86);
pub const COL_AMMO_BG: Color = COL_GEARS;
pub const COL_AMMO_FILL: Color = COL_PADDLE_CAPTURE;
pub const COL_AMMO_OUT: Color = COL_GEARS_DISABLED;
pub const COL_ENEMY: Color = Color::srgb(0.67, 0.12, 0.40);
pub const COL_ENEMY_FLASH: Color = COL_PADDLE;
pub const COL_ENEMY_PROJECTILE: Color = Color::srgb(1.0, 0.31, 0.41);

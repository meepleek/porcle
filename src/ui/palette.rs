use bevy::prelude::*;

pub const NODE_BG: Color = COL_BULLET;
pub const BUTTON_BG: Color = COL_TRANSITION_1;
pub const BUTTON_HOVERED_BG: Color = COL_TRANSITION_2;
pub const BUTTON_PRESSED_BG: Color = COL_TRANSITION_3;

pub const BUTTON_TEXT: Color = COL_BALL_FAST;
pub const LABEL_TEXT: Color = COL_BALL_FAST;
pub const HEADER_TEXT: Color = COL_BG;

// looks pretty nice
// todo: couple contrast issues
// also should probly replace the white with something slightly darker
// https://lospec.com/palette-list/citrink
pub const COL_BG: Color = Color::srgb(0.13, 0.08, 0.20);
pub const COL_LETTERBOX: Color = Color::BLACK;
pub const COL_TRANSITION_1: Color = Color::srgb(0.09, 0.43, 0.48);
pub const COL_TRANSITION_2: Color = Color::srgb(0.15, 0.30, 0.44);
pub const COL_TRANSITION_3: Color = Color::srgb(0.15, 0.14, 0.27);
pub const COL_PADDLE: Color = Color::srgb(0.27, 0.57, 0.65);
pub const COL_PADDLE_TRACKS: Color = Color::srgb(0.15, 0.30, 0.44);
pub const COL_BULLET: Color = Color::srgb(0.99, 0.96, 0.38);
pub const COL_PADDLE_REFLECT: Color = Color::srgb(0.09, 0.43, 0.48);
pub const COL_PADDLE_CAPTURED: Color = COL_GEARS_DISABLED;
pub const COL_PADDLE_CAPTURE: Color = COL_BALL;
pub const COL_BALL: Color = Color::srgb(0.55, 0.85, 0.58);
pub const COL_BALL_FAST: Color = Color::srgb(0.85, 0.89, 0.99);
pub const COL_GEARS: Color = Color::srgb(0.15, 0.30, 0.44);
pub const COL_GEARS_DISABLED: Color = Color::srgb(0.15, 0.14, 0.27);
pub const COL_AMMO_BG: Color = COL_GEARS;
pub const COL_AMMO_FILL: Color = COL_BULLET;
pub const COL_AMMO_OUT: Color = Color::srgb(0.15, 0.14, 0.27);
pub const COL_ENEMY: Color = Color::srgb(0.79, 0.38, 0.68);
pub const COL_ENEMY_FLASH: Color = COL_BALL_FAST;
pub const COL_ENEMY_PROJECTILE: Color = Color::srgb(0.92, 0.36, 0.75);

// todo: try to move a couple of colors around
// and use colors from
// https://lospec.com/palette-list/chasm
// for enemies

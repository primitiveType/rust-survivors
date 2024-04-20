use bevy::math::Vec3;
use bevy::prelude::{Color, Val};

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
pub const PADDLE_SIZE: Vec3 = Vec3::new(50.0, 50.0, 1.0);
pub const PLAYER_SPEED: f32 = 200.0;
// How close can the paddle get to the wall
pub const PADDLE_PADDING: f32 = 10.0;

pub const XP_DIAMETER: f32 = 5.;

pub const WALL_THICKNESS: f32 = 10.0;
// x coordinates
pub const LEFT_WALL: f32 = -450.;
pub const RIGHT_WALL: f32 = 450.;
// y coordinates
pub const BOTTOM_WALL: f32 = -300.;
pub const TOP_WALL: f32 = 300.;

pub const SCOREBOARD_FONT_SIZE: f32 = 40.0;
pub const SCOREBOARD_TEXT_PADDING: Val = Val::Px(20.0);

pub const BACKGROUND_COLOR: Color = Color::rgb(0.0, 0.0, 0.1);
pub const TEXT_COLOR: Color = Color::rgb(0.5, 0.5, 1.0);
pub const SCORE_COLOR: Color = Color::rgb(1.0, 0.5, 0.5);

pub const PIXEL_SCALE: f32 = 4.0;
pub const STARTING_LAYER: f32 = 100.0;
pub const BACKGROUND_PROJECTILE_LAYER: f32 = -1.0 + STARTING_LAYER;
pub const DAMAGE_TEXT_LAYER: f32 = 1.0 + ENEMY_LAYER;

pub const CORPSE_LAYER: f32 = -1.0 + ENEMY_LAYER;
pub const PLAYER_LAYER: f32 = 10.0 + STARTING_LAYER;
pub const ENEMY_LAYER: f32 = 1.0 + PLAYER_LAYER;

pub const XP_LAYER: f32 = -2.0 + STARTING_LAYER;

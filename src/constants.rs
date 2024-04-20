use bevy::prelude::{Color, Val};

// These constants are defined in `Transform` units.
// Using the default 2D camera they correspond 1:1 with screen pixels.
pub const PLAYER_SPEED: f32 = 200.0;

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

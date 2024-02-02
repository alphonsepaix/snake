use crate::game_logic::SnakeDirection;
use bevy::prelude::*;

pub const WALL_THICKNESS: f32 = 5.0;
pub const WALL_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);

pub const BACKGROUND_COLOR: Color = Color::rgb(0.05, 0.05, 0.05);
pub const TILE_SIZE: Vec2 = Vec2::new(20.0, 20.0);
pub const REFRESH_RATE: f32 = 6.0;
pub const GRID_WIDTH: usize = 17;
pub const GRID_HEIGHT: usize = 17;

pub const SCOREBOARD_FONT_SIZE: f32 = 20.0;
pub const TEXT_COLOR: Color = Color::rgb(0.0, 0.7, 0.0);

pub const SNAKE_SIZE: Vec2 = Vec2::new(17.5, 17.5);
pub const INITIAL_SNAKE_DIRECTION: SnakeDirection = SnakeDirection::Up;
pub const HEAD_COLOR: Color = Color::rgb(0.0, 1.0, 0.0);
pub const TAIL_COLOR: Color = Color::rgb(0.0, 0.4, 0.0);

pub const APPLE_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);
pub const APPLE_SIZE: Vec2 = Vec2::new(12.0, 12.0);

pub const WINDOW_PADDING: f32 = 50.0;

pub const SPLASH_SCREEN_DURATION: f32 = 2.0;

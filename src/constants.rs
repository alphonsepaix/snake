use crate::game_logic::SnakeDirection;
use bevy::prelude::*;

pub const WALL_THICKNESS: f32 = 5.0;
pub const WALL_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);

pub const BACKGROUND_COLOR: Color = Color::rgb(0.05, 0.05, 0.05);
pub const TILE_SIZE: Vec2 = Vec2::new(20.0, 20.0);
pub const REFRESH_RATE: f32 = 7.0;
pub const GRID_WIDTH: usize = 21;
pub const GRID_HEIGHT: usize = 21;

pub const SCOREBOARD_FONT_SIZE: f32 = 20.0;
pub const TEXT_COLOR: Color = Color::rgb(0.0, 0.7, 0.0);
pub const MENU_TEXT_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
pub const RESULTS_TEXT_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
pub const TEXT_BUTTON_SIZE: f32 = 40.0;
pub const BUTTON_WIDTH: f32 = 250.0;
pub const BUTTON_HEIGHT: f32 = 65.0;
pub const BUTTON_MARGIN: f32 = 20.0;
pub const MENU_TITLE_SIZE: f32 = 40.0;
pub const RESULTS_TEXT_SIZE: f32 = 30.0;

pub const SNAKE_SIZE: Vec2 = Vec2::new(17.5, 17.5);
pub const INITIAL_SNAKE_DIRECTION: SnakeDirection = SnakeDirection::Up;
pub const HEAD_COLOR: Color = Color::rgb(0.0, 1.0, 0.0);
pub const TAIL_COLOR: Color = Color::rgb(0.0, 0.4, 0.0);

pub const APPLE_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);
pub const APPLE_SIZE: Vec2 = Vec2::new(12.0, 12.0);

pub const WINDOW_PADDING: f32 = 50.0;

pub const SPLASH_SCREEN_DURATION: f32 = 1.0;
pub const RESULTS_SCREEN_DURATION: f32 = 2.0;

pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const HOVERED_PRESSED_BUTTON: Color = Color::rgb(0.25, 0.65, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

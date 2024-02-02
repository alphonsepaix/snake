pub mod constants;
pub mod game_logic;
pub mod ui;

use bevy::prelude::*;
use constants::*;
use game_logic::{PlayerInput, Scoreboard, SnakeDirection};

#[derive(Deref, DerefMut, Resource)]
pub struct AlreadyPlayed(pub bool);

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

pub fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.value.to_string();
}

pub fn handle_input(keyboard_input: Res<Input<KeyCode>>, mut player_input: ResMut<PlayerInput>) {
    let mut direction: Option<SnakeDirection> = None;
    use SnakeDirection::*;

    if keyboard_input.pressed(KeyCode::Up) || keyboard_input.pressed(KeyCode::P) {
        direction = Some(Up);
    }
    if keyboard_input.pressed(KeyCode::Down) || keyboard_input.pressed(KeyCode::I) {
        direction = Some(Down);
    }
    if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::U) {
        direction = Some(Left);
    }
    if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::E) {
        direction = Some(Right);
    }

    if direction.is_some() {
        player_input.0 = direction;
    }
}

pub fn set_menu_resolution(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    window.resolution.set(WINDOW_WIDTH, WINDOW_HEIHT);
}

pub fn set_game_resolution(mut windows: Query<&mut Window>) {
    let mut window = windows.single_mut();
    let (width, height) = (
        (GRID_WIDTH as f32 + 2.0) * TILE_SIZE.x + WINDOW_PADDING * 2.0,
        (GRID_HEIGHT as f32 + 2.0) * TILE_SIZE.y + WINDOW_PADDING * 2.0,
    );
    window.resolution.set(width, height);
}

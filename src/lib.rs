pub mod constants;
pub mod game_logic;
pub mod ui;

use bevy::prelude::*;
use game_logic::{GameEvent, PlayerInput, Scoreboard, SnakeDirection};
use std::process;

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

pub fn handle_events(mut events: EventReader<GameEvent>) {
    if !events.is_empty() {
        for event in events.read() {
            match event {
                GameEvent::GameOver(why) => println!("Game over! {}", why),
                GameEvent::GameWon => println!("You won!"),
            }
        }
        process::exit(0);
    }
}

use bevy::{prelude::*, window::WindowTheme};
use snake::constants::*;
use snake::game_logic::*;
use snake::ui::*;
use snake::*;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Snake".to_string(),
                    resolution: (
                        (GRID_WIDTH as f32 + 2.0) * TILE_SIZE.x + WINDOW_PADDING * 2.0,
                        (GRID_HEIGHT as f32 + 2.0) * TILE_SIZE.y + WINDOW_PADDING * 2.0,
                    )
                        .into(),
                    window_theme: Some(WindowTheme::Dark),
                    enabled_buttons: bevy::window::EnabledButtons {
                        maximize: false,
                        ..default()
                    },
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins((splash::SplashPlugin, menu::MenuPlugin, game::GamePlugin))
        .add_state::<GameState>()
        .add_event::<GameEvent>()
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

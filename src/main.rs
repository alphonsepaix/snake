use bevy::{prelude::*, window::WindowTheme};
use snake::constants::*;
use snake::game_logic::*;
use snake::ui::results::ResultsTimer;
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
        .add_plugins((
            splash::SplashPlugin,
            menu::MenuPlugin,
            game::GamePlugin,
            results::ResultsPlugin,
        ))
        .add_state::<GameState>()
        .add_event::<GameEvent>()
        .insert_resource(Scoreboard { value: 0 })
        .insert_resource(PlayerInput(None))
        .insert_resource(SnakeBody { body: vec![] })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(GameTimer(Timer::from_seconds(
            1.0 / REFRESH_RATE,
            TimerMode::Repeating,
        )))
        .insert_resource(ResultsTimer(Timer::from_seconds(
            RESULTS_SCREEN_DURATION,
            TimerMode::Repeating,
        )))
        .add_systems(Startup, setup)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

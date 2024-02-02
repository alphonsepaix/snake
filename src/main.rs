use bevy::{prelude::*, window::WindowTheme};
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
        .insert_resource(Scoreboard { value: 0 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(GameTimer(Timer::from_seconds(
            1.0 / REFRESH_RATE,
            TimerMode::Repeating,
        )))
        .add_event::<GameEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (
                handle_input,
                move_snake,
                check_for_collisions,
                handle_events,
            )
                .chain(),
        )
        .add_systems(Update, (update_scoreboard, bevy::window::close_on_esc))
        .run();
}

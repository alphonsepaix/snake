pub mod constants;
pub mod game_logic;
pub mod ui;

use bevy::{prelude::*, winit::WinitWindows};
use constants::*;
use game_logic::{PlayerInput, Scoreboard, SnakeDirection};
use winit::window::Icon;

#[derive(Deref, DerefMut, Resource)]
pub struct AlreadyPlayed(pub bool);

#[derive(Resource)]
pub struct AppleSound(Handle<AudioSource>);

#[derive(Resource)]
pub struct WallSound(Handle<AudioSource>);

#[derive(Resource)]
pub struct ButtonHoveredSound(Handle<AudioSource>);

#[derive(Resource)]
pub struct ButtonPressedSound(Handle<AudioSource>);

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn(AudioBundle {
        source: asset_server.load("music.ogg"),
        settings: PlaybackSettings::LOOP,
    });
    commands.insert_resource(AppleSound(asset_server.load("apple.ogg")));
    commands.insert_resource(WallSound(asset_server.load("wall.ogg")));
    commands.insert_resource(ButtonHoveredSound(asset_server.load("hovered.ogg")));
    commands.insert_resource(ButtonPressedSound(asset_server.load("pressed.ogg")));
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

pub fn set_window_icon(windows: NonSend<WinitWindows>) {
    let (rgba, width, height) = {
        let image = image::open("assets/icon_snake.png").unwrap().into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(rgba, width, height).unwrap();
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}

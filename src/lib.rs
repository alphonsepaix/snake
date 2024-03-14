pub mod constants;
pub mod logic;
pub mod ui;

use bevy::{core::FrameCount, prelude::*, winit::WinitWindows};
use constants::*;
use logic::{PlayerInput, Scoreboard, SnakeDirection};
use winit::window::Icon;

use crate::ui::game::GameMode;

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

#[derive(Component)]
pub struct MainMusic;

pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        AudioBundle {
            source: asset_server.load("music.ogg"),
            settings: PlaybackSettings::LOOP,
        },
        MainMusic,
    ));
    commands.insert_resource(AppleSound(asset_server.load("apple.ogg")));
    commands.insert_resource(WallSound(asset_server.load("wall.ogg")));
    commands.insert_resource(ButtonHoveredSound(asset_server.load("hovered.ogg")));
    commands.insert_resource(ButtonPressedSound(asset_server.load("pressed.ogg")));
}

pub fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.value.to_string();
}

pub fn handle_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_input: ResMut<PlayerInput>,
    current_state: Res<State<GameMode>>,
    mut next_state: ResMut<NextState<GameMode>>,
    music_controller: Query<&AudioSink, With<MainMusic>>,
) {
    // Toggle game state
    if keyboard_input.just_pressed(KeyCode::Space) {
        let sink = music_controller.get_single().unwrap();
        let audio_scale_factor = 3.0;
        if let GameMode::Running = current_state.get() {
            next_state.set(GameMode::Pause);
            sink.set_volume(sink.volume() / audio_scale_factor);
        } else {
            next_state.set(GameMode::Running);
            sink.set_volume(sink.volume() * audio_scale_factor);
        }
    }

    if let GameMode::Running = current_state.get() {
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

        if let Some(direction) = direction {
            player_input.push(direction);
        }
    }
}

pub fn set_window_icon(windows: NonSend<WinitWindows>) {
    let (rgba, width, height) = {
        let image = image::open("assets/icon.png").unwrap().into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(rgba, width, height).unwrap();
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}

pub fn get_window_resolution() -> (f32, f32) {
    let (width, height) = (
        (GRID_WIDTH as f32 + 2.0) * TILE_SIZE.x + WINDOW_PADDING * 2.0,
        (GRID_HEIGHT as f32 + 2.0) * TILE_SIZE.y + WINDOW_PADDING * 2.0,
    );
    (width.max(MENU_WIDTH), height.max(MENU_HEIGHT))
}

pub fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
    if frames.0 == WINDOW_VISIBLE_DELAY {
        window.single_mut().visible = true;
    }
}

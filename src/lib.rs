pub mod constants;
pub mod game_logic;

use bevy::prelude::*;
pub use constants::*;
pub use game_logic::*;
use std::process;

pub enum WallLocation {
    Top,
    Bottom,
    Left,
    Right,
}

impl WallLocation {
    pub fn size(&self) -> Vec2 {
        use WallLocation::*;
        match self {
            Top | Bottom => Vec2::new(
                GRID_WIDTH as f32 * TILE_SIZE.x + TILE_SIZE.x + WALL_THICKNESS,
                WALL_THICKNESS,
            ),
            Left | Right => Vec2::new(
                WALL_THICKNESS,
                GRID_HEIGHT as f32 * TILE_SIZE.y + TILE_SIZE.y + WALL_THICKNESS,
            ),
        }
    }

    pub fn position(&self) -> Vec2 {
        let x = ((GRID_WIDTH + 1) / 2) as f32 * TILE_SIZE.x;
        let y = ((GRID_HEIGHT + 1) / 2) as f32 * TILE_SIZE.y;
        match self {
            WallLocation::Top => Vec2::new(0.0, y),
            WallLocation::Bottom => Vec2::new(0.0, -y),
            WallLocation::Left => Vec2::new(-x, 0.0),
            WallLocation::Right => Vec2::new(x, 0.0),
        }
    }
}

#[derive(Bundle)]
struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl WallBundle {
    fn new(location: WallLocation) -> Self {
        WallBundle {
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: location.position().extend(0.0),
                    scale: location.size().extend(0.0),
                    ..default()
                },
                sprite: Sprite {
                    color: WALL_COLOR,
                    ..default()
                },
                ..default()
            },
            collider: Collider,
        }
    }
}

pub fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // The walls
    commands.spawn(WallBundle::new(WallLocation::Top));
    commands.spawn(WallBundle::new(WallLocation::Bottom));
    commands.spawn(WallBundle::new(WallLocation::Left));
    commands.spawn(WallBundle::new(WallLocation::Right));

    // The snake
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                scale: SNAKE_SIZE.extend(0.0),
                ..default()
            },
            sprite: Sprite {
                color: HEAD_COLOR,
                ..default()
            },
            ..default()
        },
        Head,
        Tail,
        Movement(INITIAL_SNAKE_DIRECTION),
    ));

    commands.insert_resource(SnakeBody { body: vec![] });
    commands.insert_resource(PlayerInput(None));

    // A first apple
    let location = loop {
        let loc = gen_apple_location();
        if loc != Vec2::splat(0.0) {
            break loc;
        }
    };
    spawn_apple(&mut commands, location);

    // The scoreboard
    let padding = (WINDOW_PADDING - SCOREBOARD_FONT_SIZE) / 2.0;
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "Apples eaten: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                color: TEXT_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(padding),
            left: Val::Px(padding),
            ..default()
        }),
    );
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

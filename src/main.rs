#![allow(clippy::type_complexity)]

use bevy::{prelude::*, sprite::collide_aabb::collide, window::WindowTheme};
use rand::{thread_rng, Rng};
use std::{
    ops::{Deref, DerefMut},
    process,
};

const BACKGROUND_COLOR: Color = Color::rgb(0.05, 0.05, 0.05);
const TILE_SIZE: Vec2 = Vec2::new(20.0, 20.0);
const REFRESH_RATE: f32 = 6.0;
const GRID_WIDTH: usize = 17;
const GRID_HEIGHT: usize = 17;

const SCOREBOARD_FONT_SIZE: f32 = 20.0;
const TEXT_COLOR: Color = Color::rgb(0.0, 0.7, 0.0);

const SNAKE_SIZE: Vec2 = Vec2::new(17.5, 17.5);
const INITIAL_SNAKE_DIRECTION: SnakeDirection = SnakeDirection::Up;
const HEAD_COLOR: Color = Color::rgb(0.0, 1.0, 0.0);
const TAIL_COLOR: Color = Color::rgb(0.0, 0.4, 0.0);

const APPLE_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);
const APPLE_SIZE: Vec2 = Vec2::new(12.0, 12.0);

const WINDOW_PADDING: f32 = 50.0;

const WALL_THICKNESS: f32 = 5.0;
const WALL_COLOR: Color = Color::rgb(0.5, 0.5, 0.5);

#[derive(Resource, Deref, DerefMut)]
struct GameTimer(Timer);

#[derive(Event)]
enum GameEvent {
    GameOver(String),
    GameWon,
}

#[derive(Debug, Resource)]
struct Scoreboard {
    value: usize,
}

#[derive(Component)]
struct Head;

#[derive(Debug, Resource)]
struct SnakeBody {
    body: Vec<Entity>,
}

impl Deref for SnakeBody {
    type Target = Vec<Entity>;

    fn deref(&self) -> &Self::Target {
        &self.body
    }
}

impl DerefMut for SnakeBody {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.body
    }
}

#[derive(Deref, DerefMut, Resource)]
struct PlayerInput(Option<SnakeDirection>);

#[derive(Component)]
struct Tail;

#[derive(Component, Deref, DerefMut)]
struct Movement(SnakeDirection);

#[derive(Clone, Copy)]
enum SnakeDirection {
    Left,
    Right,
    Up,
    Down,
}

impl From<SnakeDirection> for Vec2 {
    fn from(value: SnakeDirection) -> Self {
        match value {
            SnakeDirection::Left => Vec2::new(-1.0, 0.0),
            SnakeDirection::Right => Vec2::new(1.0, 0.0),
            SnakeDirection::Up => Vec2::new(0.0, 1.0),
            SnakeDirection::Down => Vec2::new(0.0, -1.0),
        }
    }
}

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Apple;

fn spawn_apple(commands: &mut Commands, loc: Vec2) {
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: loc.extend(0.0),
                scale: APPLE_SIZE.extend(0.0),
                ..default()
            },
            sprite: Sprite {
                color: APPLE_COLOR,
                ..default()
            },
            ..default()
        },
        Apple,
        Collider,
    ));
}

enum WallLocation {
    Top,
    Bottom,
    Left,
    Right,
}

impl WallLocation {
    fn size(&self) -> Vec2 {
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

    fn position(&self) -> Vec2 {
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

fn setup(mut commands: Commands) {
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

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.value.to_string();
}

fn handle_input(keyboard_input: Res<Input<KeyCode>>, mut player_input: ResMut<PlayerInput>) {
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

fn move_snake(
    time: Res<Time>,
    mut timer: ResMut<GameTimer>,
    player_input: Res<PlayerInput>,
    body: Res<SnakeBody>,
    mut head: Query<(&mut Transform, &mut Movement), With<Head>>,
    mut tail: Query<(&mut Transform, &Movement), (With<Tail>, Without<Head>)>,
) {
    let (mut snake_transform, mut snake_velocity) = head.single_mut();

    if timer.tick(time.delta()).just_finished() {
        if let Some(next_direction) = player_input.0 {
            use SnakeDirection::*;
            match (snake_velocity.0, next_direction) {
                (Left, Right) | (Right, Left) | (Up, Down) | (Down, Up) => (),
                _ => snake_velocity.0 = next_direction,
            }
        }
        // For each body segment, set the transform of one segment to match the transform
        // of the segment above
        if !body.is_empty() {
            for i in (0..body.len()).rev() {
                let next_trans = if i == 0 {
                    *snake_transform
                } else {
                    *tail.component::<Transform>(body[i - 1])
                };
                let mut trans = tail.component_mut::<Transform>(body[i]);
                *trans = next_trans;
            }
        }

        let direction: Vec2 = Into::<Vec2>::into(snake_velocity.0);
        snake_transform.translation.x += direction.x * TILE_SIZE.x;
        snake_transform.translation.y += direction.y * TILE_SIZE.y;
        snake_transform.translation.x =
            (snake_transform.translation.x / TILE_SIZE.x).round() * TILE_SIZE.x;
        snake_transform.translation.y =
            (snake_transform.translation.y / TILE_SIZE.y).round() * TILE_SIZE.y;
    }
}

fn check_for_collisions(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut body: ResMut<SnakeBody>,
    mut events: EventWriter<GameEvent>,
    snake_query: Query<&mut Transform, With<Head>>,
    collider_query: Query<
        (Entity, &Transform, Option<&Apple>, Option<&Tail>),
        (With<Collider>, Without<Head>),
    >,
    tail: Query<&Transform, (With<Tail>, Without<Head>)>,
) {
    let snake_transform = snake_query.single();

    for (collider_entity, transform, maybe_apple, maybe_tail) in &collider_query {
        let collision = collide(
            snake_transform.translation,
            TILE_SIZE,
            transform.translation,
            transform.scale.truncate(),
        );
        if collision.is_some() {
            if maybe_apple.is_some() {
                // Handle collision with an apple
                commands.entity(collider_entity).despawn();
                let apple_loc = loop {
                    let location = gen_apple_location();
                    if collider_query
                        .iter()
                        .all(|query| query.1.translation.truncate() != location)
                    {
                        break location;
                    }
                };
                spawn_apple(&mut commands, apple_loc);
                scoreboard.value += 1;

                let transform = if body.is_empty() {
                    *snake_transform
                } else {
                    let tail_id = body.last().unwrap();
                    *tail.component::<Transform>(*tail_id)
                };
                let new_tail = commands
                    .spawn((
                        SpriteBundle {
                            transform,
                            sprite: Sprite {
                                color: TAIL_COLOR,
                                ..default()
                            },
                            ..default()
                        },
                        Tail,
                        Collider,
                        Movement(INITIAL_SNAKE_DIRECTION),
                    ))
                    .id();
                body.push(new_tail);
                if body.len() == GRID_HEIGHT * GRID_WIDTH - 1 {
                    events.send(GameEvent::GameWon);
                }
            } else if maybe_tail.is_some() {
                if body.len() > 1 {
                    // Collision with tail
                    events.send(GameEvent::GameOver("You hit your tail!".into()));
                }
            } else {
                // Collision with walls
                events.send(GameEvent::GameOver("You hit a wall!".into()));
            }
        }
    }
}

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

fn handle_events(mut events: EventReader<GameEvent>) {
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

fn gen_apple_location() -> Vec2 {
    let mut rng = thread_rng();

    let x = (GRID_WIDTH - 1) as f32 / 2.0 * TILE_SIZE.x;
    let y = (GRID_HEIGHT - 1) as f32 / 2.0 * TILE_SIZE.y;

    let mut x = rng.gen_range(-x..=x);
    let mut y = rng.gen_range(-y..=y);
    x = (x / TILE_SIZE.x).round() * TILE_SIZE.x;
    y = (y / TILE_SIZE.y).round() * TILE_SIZE.y;

    Vec2::new(x, y)
}

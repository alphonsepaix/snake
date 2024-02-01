#![allow(clippy::type_complexity)]

use bevy::{prelude::*, sprite::collide_aabb::collide, window::WindowTheme};
use rand::{thread_rng, Rng};
use std::ops::{Deref, DerefMut};

const BACKGROUND_COLOR: Color = Color::rgb(0.05, 0.05, 0.05);
const TILE_SIZE: Vec2 = Vec2::new(20.0, 20.0);
const REFRESH_RATE: f32 = 8.0;
const GRID_WIDTH: usize = 21;
const GRID_HEIGHT: usize = 21;

const SCOREBOARD_FONT_SIZE: f32 = 42.0;
const TEXT_COLOR: Color = Color::rgb(0.0, 1.0, 0.0);

const SNAKE_SIZE: Vec2 = Vec2::new(17.5, 17.5);
const INITIAL_SNAKE_DIRECTION: SnakeDirection = SnakeDirection::Up;
const HEAD_COLOR: Color = Color::rgb(0.0, 1.0, 0.0);
const TAIL_COLOR: Color = Color::rgb(0.0, 0.8, 0.0);

const APPLE_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);
const APPLE_SIZE: Vec2 = Vec2::new(12.0, 12.0);

const WINDOW_WIDTH: f32 = 1100.0;
const WINDOW_HEIGHT: f32 = 800.0;

const LEFT_WALL: f32 = -400.0;
const RIGHT_WALL: f32 = 400.0;
const TOP_WALL: f32 = 300.0;
const BOTTOM_WALL: f32 = -300.0;
const WALL_THICKNESS: f32 = 8.0;
const WALL_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

#[derive(Resource, Deref, DerefMut)]
struct GameTimer(Timer);

#[derive(Event, Default)]
struct CollisionEvent;

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
        let arena_width = RIGHT_WALL - LEFT_WALL;
        let arena_height = TOP_WALL - BOTTOM_WALL;
        match self {
            Top | Bottom => Vec2::new(arena_width + WALL_THICKNESS, WALL_THICKNESS),
            Left | Right => Vec2::new(WALL_THICKNESS, arena_height + WALL_THICKNESS),
        }
    }

    fn position(&self) -> Vec2 {
        match self {
            WallLocation::Top => Vec2::new(0.0, TOP_WALL),
            WallLocation::Bottom => Vec2::new(0.0, BOTTOM_WALL),
            WallLocation::Left => Vec2::new(LEFT_WALL, 0.0),
            WallLocation::Right => Vec2::new(RIGHT_WALL, 0.0),
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
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "SCORE - ",
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
            top: Val::Px(10.0),
            left: Val::Px(10.0),
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

    if keyboard_input.pressed(KeyCode::Up) {
        direction = Some(Up);
    }
    if keyboard_input.pressed(KeyCode::Down) {
        direction = Some(Down);
    }
    if keyboard_input.pressed(KeyCode::Left) {
        direction = Some(Left);
    }
    if keyboard_input.pressed(KeyCode::Right) {
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
    // let mut direction: Option<SnakeDirection> = None;
    use SnakeDirection::*;

    // if keyboard_input.pressed(KeyCode::Up) {
    //     direction = Some(Up);
    // }
    // if keyboard_input.pressed(KeyCode::Down) {
    //     direction = Some(Down);
    // }
    // if keyboard_input.pressed(KeyCode::Left) {
    //     direction = Some(Left);
    // }
    // if keyboard_input.pressed(KeyCode::Right) {
    //     direction = Some(Right);
    // }

    let (mut snake_transform, mut snake_velocity) = head.single_mut();

    // if let Some(next_direction) = direction {
    //     match (snake_velocity.0, next_direction) {
    //         (Left, Right) | (Right, Left) | (Up, Down) | (Down, Up) => (),
    //         _ => snake_velocity.0 = next_direction,
    //     }
    // }

    if timer.tick(time.delta()).just_finished() {
        if let Some(next_direction) = player_input.0 {
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
                    *tail.component::<Transform>(body.body[i - 1])
                };
                let mut trans = tail.component_mut::<Transform>(body.body[i]);
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
            } else if maybe_tail.is_some() {
                if body.len() > 1 {
                    // Collision with tail
                    info!("TAIL COLLISION");
                }
            } else {
                // Collision with walls
                info!("WALL COLLISION");
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Snake".to_string(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                window_theme: Some(WindowTheme::Dark),
                enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .insert_resource(Scoreboard { value: 0 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(GameTimer(Timer::from_seconds(
            1.0 / REFRESH_RATE,
            TimerMode::Repeating,
        )))
        .add_event::<CollisionEvent>()
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (handle_input, move_snake, check_for_collisions).chain(),
        )
        .add_systems(Update, (update_scoreboard, bevy::window::close_on_esc))
        .run();
}

fn gen_apple_location() -> Vec2 {
    let mut rng = thread_rng();

    let padding = 15.0;
    let x_min = LEFT_WALL + WALL_THICKNESS / 2.0 + padding;
    let x_max = RIGHT_WALL - WALL_THICKNESS / 2.0 - padding;
    let y_min = BOTTOM_WALL + WALL_THICKNESS / 2.0 + padding;
    let y_max = TOP_WALL - WALL_THICKNESS / 2.0 - padding;

    let mut x = rng.gen_range(x_min..=x_max);
    let mut y = rng.gen_range(y_min..=y_max);
    x = (x / TILE_SIZE.x).round() * TILE_SIZE.x;
    y = (y / TILE_SIZE.y).round() * TILE_SIZE.y;

    Vec2::new(x, y)
}

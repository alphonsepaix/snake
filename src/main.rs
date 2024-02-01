#![allow(clippy::type_complexity)]

use bevy::{prelude::*, sprite::collide_aabb::collide, window::WindowTheme};
use rand::{thread_rng, Rng};

const SCOREBOARD_FONT_SIZE: f32 = 42.0;

const TEXT_COLOR: Color = Color::rgb(0.0, 1.0, 0.0);
const BACKGROUND_COLOR: Color = Color::rgb(0.15, 0.15, 0.15);
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

const SNAKE_SIZE: Vec3 = Vec3::new(20.0, 20.0, 0.0);
const SNAKE_COLOR: Color = Color::rgb(0.0, 1.0, 0.0);
const INITIAL_SNAKE_SPEED: f32 = 200.0;
const INITIAL_SNAKE_DIRECTION: SnakeDirection = SnakeDirection::Up;

const APPLE_SIZE: Vec3 = Vec3::new(20.0, 20.0, 0.0);
const APPLE_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);

const WINDOW_WIDTH: f32 = 1100.0;
const WINDOW_HEIGHT: f32 = 800.0;

const OFFSET: f32 = 50.0;
const LEFT_WALL: f32 = -500.0;
const RIGHT_WALL: f32 = 500.0;
const TOP_WALL: f32 = 250.0;
const BOTTOM_WALL: f32 = -350.0;
const WALL_THICKNESS: f32 = 15.0;

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

#[derive(Component)]
struct Snake;

#[derive(Component)]
struct Velocity {
    speed: f32,
    direction: SnakeDirection,
}

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

fn spawn_apple(commands: &mut Commands) {
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: gen_apple_location().extend(0.0),
                scale: APPLE_SIZE,
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
            WallLocation::Left => Vec2::new(LEFT_WALL, -OFFSET),
            WallLocation::Right => Vec2::new(RIGHT_WALL, -OFFSET),
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
                    scale: location.size().extend(1.0),
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
                translation: Vec3::new(0.0, -OFFSET, 0.0),
                scale: SNAKE_SIZE,
                ..default()
            },
            sprite: Sprite {
                color: SNAKE_COLOR,
                ..default()
            },
            ..default()
        },
        Head,
        Snake,
        Velocity {
            speed: INITIAL_SNAKE_SPEED,
            direction: INITIAL_SNAKE_DIRECTION,
        },
    ));

    commands.insert_resource(SnakeBody { body: vec![] });

    // A first apple
    spawn_apple(&mut commands);

    // The scoreboard
    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "SCORE -- ",
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
            top: Val::Px(50.0),
            left: Val::Px(50.0),
            ..default()
        }),
    );
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.value.to_string();
}

fn move_snake(
    keyboard_input: Res<Input<KeyCode>>,
    mut head: Query<(&mut Transform, &mut Velocity), With<Head>>,
    mut body: Query<(&mut Transform, &Velocity), (With<Snake>, Without<Head>)>,
    body_id: Res<SnakeBody>,
    time: Res<Time>,
) {
    let (mut snake_transform, mut snake_velocity) = head.single_mut();

    use SnakeDirection::*;
    let mut direction: Option<SnakeDirection> = None;

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

    if let Some(next_direction) = direction {
        match (snake_velocity.direction, next_direction) {
            (Left, Right) | (Right, Left) | (Up, Down) | (Down, Up) => (),
            _ => snake_velocity.direction = next_direction,
        }
    }

    let Velocity { speed, direction } = *snake_velocity;
    let direction: Vec2 = direction.into();
    snake_transform.translation.x += direction.x * speed * time.delta_seconds();
    snake_transform.translation.y += direction.y * speed * time.delta_seconds();

    // For each body segment, set the transform of one segment to match the transform
    // of the segment above
    if !body_id.body.is_empty() {
        for i in (0..body_id.body.len()).rev() {
            let next_trans = if i == 0 {
                *snake_transform
            } else {
                *body.component::<Transform>(body_id.body[i - 1])
            };
            let mut trans = body.component_mut::<Transform>(body_id.body[i]);
            *trans = next_trans;
        }
    }
}

fn check_for_collisions(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut snake_query: Query<&mut Transform, With<Head>>,
    collider_query: Query<(Entity, &Transform, Option<&Apple>), (With<Collider>, Without<Head>)>,
    mut snake: ResMut<SnakeBody>,
    body: Query<&Transform, (With<Snake>, Without<Head>)>,
) {
    let mut snake_transform = snake_query.single_mut();
    let Vec2 { x, y } = snake_transform.translation.truncate();

    for (collider_entity, transform, maybe_apple) in &collider_query {
        let collision = collide(
            snake_transform.translation,
            SNAKE_SIZE.truncate(),
            transform.translation,
            transform.scale.truncate(),
        );
        if let Some(collision) = collision {
            if maybe_apple.is_none() {
                // Check collision with walls
                let arena_width = RIGHT_WALL - LEFT_WALL - WALL_THICKNESS - SNAKE_SIZE.x;
                let arena_height = TOP_WALL - BOTTOM_WALL - WALL_THICKNESS - SNAKE_SIZE.y;
                let new_pos = match collision {
                    bevy::sprite::collide_aabb::Collision::Right => Vec2::new(x + arena_width, y),
                    bevy::sprite::collide_aabb::Collision::Left => Vec2::new(x - arena_width, y),
                    bevy::sprite::collide_aabb::Collision::Bottom => Vec2::new(x, y - arena_height),
                    bevy::sprite::collide_aabb::Collision::Top => Vec2::new(x, y + arena_height),
                    bevy::sprite::collide_aabb::Collision::Inside => {
                        panic!();
                    }
                };
                snake_transform.translation = new_pos.extend(0.0);
            } else {
                // Handle collision with an apple
                commands.entity(collider_entity).despawn();
                spawn_apple(&mut commands);
                scoreboard.value += 1;

                let transform = if snake.body.is_empty() {
                    *snake_transform
                } else {
                    let tail_id = snake.body.last().unwrap();
                    *body.component::<Transform>(*tail_id)
                };
                let new_tail = commands
                    .spawn((
                        SpriteBundle {
                            transform,
                            sprite: Sprite {
                                color: SNAKE_COLOR,
                                ..default()
                            },
                            ..default()
                        },
                        Snake,
                        Velocity {
                            speed: INITIAL_SNAKE_SPEED,
                            direction: INITIAL_SNAKE_DIRECTION,
                        },
                    ))
                    .id();

                snake.body.push(new_tail);
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
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (move_snake, check_for_collisions).chain())
        .add_systems(Update, (update_scoreboard, bevy::window::close_on_esc))
        .run();
}

fn gen_apple_location() -> Vec2 {
    let mut rng = thread_rng();

    let padding = 15.0;
    let x_min = LEFT_WALL + WALL_THICKNESS / 2.0 + padding;
    let x_max = RIGHT_WALL - WALL_THICKNESS / 2.0 - padding;
    let y_min = BOTTOM_WALL + WALL_THICKNESS / 2.0 + padding + OFFSET;
    let y_max = TOP_WALL - WALL_THICKNESS / 2.0 - padding - OFFSET;

    let x = rng.gen_range(x_min..=x_max);
    let y = rng.gen_range(y_min..=y_max);

    Vec2::new(x, y)
}

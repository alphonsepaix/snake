#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use crate::ui::game::OnGameScreen;
use crate::{constants::*, ui::GameState};
use crate::{AppleSound, WallSound};
use bevy::{prelude::*, sprite::collide_aabb::collide};
use rand::{thread_rng, Rng};
use std::ops::{Deref, DerefMut};

#[derive(Resource, Deref, DerefMut)]
pub struct GameTimer(pub Timer);

#[derive(Event)]
pub enum GameEvent {
    GameOver(String),
    GameWon,
}

#[derive(Debug, Resource)]
pub struct Scoreboard {
    pub value: usize,
}

#[derive(Component)]
pub struct Head;

#[derive(Debug, Resource)]
pub struct SnakeBody {
    pub body: Vec<Entity>,
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
pub struct PlayerInput(pub Vec<SnakeDirection>);

#[derive(Component)]
pub struct Tail;

#[derive(Component, Deref, DerefMut)]
pub struct Movement(pub SnakeDirection);

#[derive(Clone, Copy)]
pub enum SnakeDirection {
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
pub struct Collider;

#[derive(Component)]
pub struct Apple;

pub fn gen_apple_location() -> Vec2 {
    let mut rng = thread_rng();

    let x = (GRID_WIDTH - 1) as f32 / 2.0 * TILE_SIZE.x;
    let y = (GRID_HEIGHT - 1) as f32 / 2.0 * TILE_SIZE.y;

    let mut x = rng.gen_range(-x..=x);
    let mut y = rng.gen_range(-y..=y);
    x = (x / TILE_SIZE.x).round() * TILE_SIZE.x;
    y = (y / TILE_SIZE.y).round() * TILE_SIZE.y;

    Vec2::new(x, y)
}

pub fn check_for_collisions(
    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
    mut body: ResMut<SnakeBody>,
    mut game_state: ResMut<NextState<GameState>>,
    mut events: EventWriter<GameEvent>,
    snake_query: Query<&mut Transform, With<Head>>,
    collider_query: Query<
        (Entity, &Transform, Option<&Apple>, Option<&Tail>),
        (With<Collider>, Without<Head>),
    >,
    tail: Query<&Transform, (With<Tail>, Without<Head>)>,
    apple_sound: Res<AppleSound>,
    wall_sound: Res<WallSound>,
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
                play_collision_sound(&mut commands, apple_sound.0.clone());

                // Spawn a new apple
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
                        OnGameScreen,
                    ))
                    .id();

                body.push(new_tail);

                if body.len() == GRID_HEIGHT * GRID_WIDTH - 1 {
                    events.send(GameEvent::GameWon);
                    game_state.set(GameState::Results);
                }
            } else if maybe_tail.is_some() {
                if body.len() > 1 {
                    // Collision with tail
                    play_collision_sound(&mut commands, wall_sound.0.clone());
                    events.send(GameEvent::GameOver("You hit your tail!".into()));
                    game_state.set(GameState::Results);
                }
            } else {
                // Collision with a wall
                play_collision_sound(&mut commands, wall_sound.0.clone());
                game_state.set(GameState::Results);
                events.send(GameEvent::GameOver("You hit a wall!".into()));
            }
        }
    }
}

pub fn move_snake(
    time: Res<Time>,
    mut timer: ResMut<GameTimer>,
    mut player_input: ResMut<PlayerInput>,
    body: Res<SnakeBody>,
    mut head: Query<(&mut Transform, &mut Movement), With<Head>>,
    mut tail: Query<(&mut Transform, &Movement), (With<Tail>, Without<Head>)>,
) {
    let (mut snake_transform, mut snake_velocity) = head.single_mut();

    if timer.tick(time.delta()).just_finished() {
        use SnakeDirection::*;
        for next_direction in player_input.iter().rev() {
            match (snake_velocity.0, next_direction) {
                (Left, Right) | (Right, Left) | (Up, Down) | (Down, Up) => continue,
                _ => {
                    snake_velocity.0 = *next_direction;
                    break;
                }
            }
        }
        player_input.clear();
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

pub fn spawn_apple(commands: &mut Commands, loc: Vec2) {
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
        OnGameScreen,
    ));
}

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
pub struct WallBundle {
    sprite_bundle: SpriteBundle,
    collider: Collider,
}

impl WallBundle {
    pub fn new(location: WallLocation) -> Self {
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

fn play_collision_sound(commands: &mut Commands, source: Handle<AudioSource>) {
    commands.spawn(AudioBundle {
        source,
        settings: PlaybackSettings::DESPAWN,
    });
}

use bevy::prelude::*;

const SCOREBOARD_FONT_SIZE: f32 = 50.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(10.0);

const TEXT_COLOR: Color = Color::rgb(1.0, 1.0, 0.0);

const BACKGROUND_COLOR: Color = Color::rgb(0.3, 0.2, 0.5);

const SNAKE_SIZE: Vec3 = Vec3::new(20.0, 20.0, 0.0);
const SNAKE_COLOR: Color = Color::rgb(1.0, 0.0, 0.0);
const INITIAL_SNAKE_SPEED: f32 = 300.0;
const INITIAL_SNAKE_DIRECTION: SnakeDirection = SnakeDirection::Left;

#[derive(Debug, Resource)]
struct Scoreboard {
    value: usize,
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

impl Into<Vec2> for SnakeDirection {
    fn into(self) -> Vec2 {
        match self {
            SnakeDirection::Left => Vec2::new(-1.0, 0.0),
            SnakeDirection::Right => Vec2::new(1.0, 0.0),
            SnakeDirection::Up => Vec2::new(0.0, 1.0),
            SnakeDirection::Down => Vec2::new(0.0, -1.0),
        }
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    // The snake.
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(0.0, 200.0, 0.0),
                scale: SNAKE_SIZE,
                ..default()
            },
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
    ));

    commands.spawn(
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
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
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
    );
}

fn update_scoreboard(scoreboard: Res<Scoreboard>, mut query: Query<&mut Text>) {
    let mut text = query.single_mut();
    text.sections[1].value = scoreboard.value.to_string();
}

fn update_score(mouse_input: Res<Input<MouseButton>>, mut scoreboard: ResMut<Scoreboard>) {
    if mouse_input.pressed(MouseButton::Left) {
        scoreboard.value += 1;
    } else if mouse_input.pressed(MouseButton::Right) {
        scoreboard.value = scoreboard.value.saturating_sub(1);
    } else if mouse_input.pressed(MouseButton::Middle) {
        scoreboard.value = 0;
    }
}

fn move_snake(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Snake>>,
    time: Res<Time>,
) {
    let mut direction: Option<SnakeDirection> = None;
    use SnakeDirection::*;
    if keyboard_input.pressed(KeyCode::P) {
        direction = Some(Up);
    }
    if keyboard_input.pressed(KeyCode::I) {
        direction = Some(Down);
    }
    if keyboard_input.pressed(KeyCode::U) {
        direction = Some(Left);
    }
    if keyboard_input.pressed(KeyCode::E) {
        direction = Some(Right);
    }

    let mut snake = query.single_mut();
    if let Some(direction) = direction {
        let current_direction = snake.1.direction;
        match current_direction {
            Left => !matches!(direction, Right),
            Right => !matches!(direction, Left),
            Up => !matches!(direction, Down),
            Down => !matches!(direction, Up),
        }
        .then(|| snake.1.direction = direction);
    }

    let speed = snake.1.speed;
    let direction: Vec2 = snake.1.direction.into();
    snake.0.translation.x += direction.x * speed * time.delta_seconds();
    snake.0.translation.y += direction.y * speed * time.delta_seconds();
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Scoreboard { value: 0 })
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (update_score, move_snake))
        .add_systems(Update, (update_scoreboard, bevy::window::close_on_esc))
        .run();
}

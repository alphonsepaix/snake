#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use crate::{get_window_resolution, logic::WallLocation};

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Splash,
    Menu,
    Game,
    Results,
}

pub mod splash {
    use super::{despawn_screen, GameState};
    use crate::{constants::SPLASH_SCREEN_DURATION, MENU_WIDTH};
    use bevy::prelude::*;

    pub struct SplashPlugin;

    impl Plugin for SplashPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(OnEnter(GameState::Splash), splash_setup)
                .add_systems(Update, countdown.run_if(in_state(GameState::Splash)))
                .add_systems(OnExit(GameState::Splash), despawn_screen::<OnSplashScreen>);
        }
    }

    #[derive(Component)]
    struct OnSplashScreen;

    #[derive(Resource, Deref, DerefMut)]
    struct SplashTimer(Timer);

    fn splash_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        let icon = asset_server.load("icon.png");
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    ..default()
                },
                OnSplashScreen,
            ))
            .with_children(|parent| {
                parent.spawn(ImageBundle {
                    style: Style {
                        width: Val::Px(MENU_WIDTH),
                        ..default()
                    },
                    image: UiImage::new(icon),
                    ..default()
                });
            });
        commands.insert_resource(SplashTimer(Timer::from_seconds(
            SPLASH_SCREEN_DURATION,
            TimerMode::Once,
        )));
    }

    fn countdown(
        mut game_state: ResMut<NextState<GameState>>,
        time: Res<Time>,
        mut timer: ResMut<SplashTimer>,
    ) {
        if timer.tick(time.delta()).finished() {
            game_state.set(GameState::Menu);
        }
    }
}

pub mod game {
    use super::despawn_screen;
    use super::get_scoreboard_position;
    use super::GameState;
    use crate::constants::*;
    use crate::logic::*;
    use crate::*;
    use bevy::prelude::*;

    #[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
    pub enum GameMode {
        #[default]
        Running,
        Pause,
    }

    pub struct GamePlugin;

    impl Plugin for GamePlugin {
        fn build(&self, app: &mut App) {
            app.add_state::<GameMode>()
                .add_systems(OnEnter(GameState::Game), game_setup)
                .add_systems(Update, handle_input.run_if(in_state(GameState::Game)))
                .add_systems(
                    FixedUpdate,
                    (move_snake, check_for_collisions, update_scoreboard)
                        .chain()
                        .run_if(in_state(GameState::Game))
                        .run_if(in_state(GameMode::Running)),
                )
                .add_systems(OnEnter(GameMode::Pause), pause_setup)
                .add_systems(OnExit(GameMode::Pause), despawn_screen::<OnPauseScreen>)
                .add_systems(
                    OnExit(GameState::Game),
                    (despawn_screen::<OnGameScreen>, reset_state),
                );
        }
    }

    #[derive(Component)]
    pub struct OnGameScreen;

    pub fn game_setup(
        mut commands: Commands,
        mut already_played: ResMut<AlreadyPlayed>,
        asset_server: Res<AssetServer>,
    ) {
        if !already_played.0 {
            already_played.0 = true;
        }

        // The snake
        commands.spawn((
            SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 1.0),
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
            OnGameScreen,
            Movement(INITIAL_SNAKE_DIRECTION),
        ));

        // The walls
        commands.spawn((WallBundle::new(WallLocation::Top), OnGameScreen));
        commands.spawn((WallBundle::new(WallLocation::Bottom), OnGameScreen));
        commands.spawn((WallBundle::new(WallLocation::Left), OnGameScreen));
        commands.spawn((WallBundle::new(WallLocation::Right), OnGameScreen));

        // A first apple
        let location = loop {
            let loc = gen_apple_location();
            if loc != Vec2::splat(0.0) {
                break loc;
            }
        };
        spawn_apple(&mut commands, location);

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(get_scoreboard_position()),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::End,
                        ..default()
                    },
                    ..default()
                },
                OnGameScreen,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::End,
                            ..default()
                        },
                        background_color: Color::BLACK.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_sections([
                            TextSection::new(
                                "Score = ".to_uppercase(),
                                TextStyle {
                                    font_size: SCOREBOARD_FONT_SIZE,
                                    color: TEXT_COLOR,
                                    font: asset_server.load("font.ttf"),
                                },
                            ),
                            TextSection::from_style(TextStyle {
                                font_size: SCOREBOARD_FONT_SIZE,
                                color: TEXT_COLOR,
                                font: asset_server.load("font.ttf"),
                                ..default()
                            }),
                        ]));
                    });
            });
    }

    fn reset_state(
        mut scoreboard: ResMut<Scoreboard>,
        mut player_input: ResMut<PlayerInput>,
        mut snake_body: ResMut<SnakeBody>,
        mut timer: ResMut<GameTimer>,
    ) {
        scoreboard.value = 0;
        snake_body.clear();
        player_input.0 = vec![];
        timer.reset();
    }

    #[derive(Component)]
    pub struct OnPauseScreen;

    pub fn pause_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: BackgroundColor(Color::rgba(0.0, 0.0, 0.0, 0.7)),
                    ..default()
                },
                OnPauseScreen,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_sections([
                    TextSection::new(
                        "Pause".to_uppercase(),
                        TextStyle {
                            font_size: SCOREBOARD_FONT_SIZE,
                            color: TEXT_COLOR,
                            font: asset_server.load("font.ttf"),
                        },
                    ),
                    TextSection::from_style(TextStyle {
                        font_size: SCOREBOARD_FONT_SIZE,
                        color: TEXT_COLOR,
                        ..default()
                    }),
                ]));
            });
    }
}

pub mod menu {
    use super::results::ResultsTimer;
    use super::{despawn_screen, GameState};
    use crate::{constants::*, AlreadyPlayed, ButtonHoveredSound, ButtonPressedSound};
    use bevy::app::AppExit;
    use bevy::prelude::*;

    pub struct MenuPlugin;

    impl Plugin for MenuPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(OnEnter(GameState::Menu), menu_setup)
                .add_systems(
                    Update,
                    (menu_action, button_system).run_if(in_state(GameState::Menu)),
                )
                .add_systems(OnExit(GameState::Menu), despawn_screen::<OnMenuScreen>);
        }
    }

    #[derive(Component)]
    pub struct OnMenuScreen;

    #[derive(Component)]
    pub struct SelectedOption;

    #[derive(Component)]
    enum MenuButtonAction {
        Play,
        Quit,
    }

    pub fn menu_setup(
        mut commands: Commands,
        mut timer: ResMut<ResultsTimer>,
        asset_server: Res<AssetServer>,
        already_played: Res<AlreadyPlayed>,
    ) {
        let button_style = Style {
            width: Val::Px(BUTTON_WIDTH),
            height: Val::Px(BUTTON_HEIGHT),
            margin: UiRect::all(Val::Px(BUTTON_MARGIN)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_text_style = TextStyle {
            font_size: TEXT_BUTTON_SIZE,
            color: Color::WHITE,
            font: asset_server.load("font.ttf"),
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                },
                OnMenuScreen,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::BLACK.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        // Game name
                        parent.spawn(
                            TextBundle::from_section(
                                "Snake".to_uppercase(),
                                TextStyle {
                                    font_size: MENU_TITLE_SIZE,
                                    color: MENU_TEXT_COLOR,
                                    font: asset_server.load("font.ttf"),
                                },
                            )
                            .with_style(Style {
                                margin: UiRect::all(Val::Px(50.0)),
                                ..default()
                            }),
                        );

                        let play_button_text = if already_played.0 { "Replay" } else { "Play" };
                        // Play button
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Play,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    play_button_text.to_uppercase(),
                                    button_text_style.clone(),
                                ));
                            });

                        // Quit button
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style,
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Quit,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "Quit".to_uppercase(),
                                    button_text_style,
                                ));
                            });
                    });
            });

        timer.reset();
    }

    fn button_system(
        mut commands: Commands,
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
            (Changed<Interaction>, With<Button>),
        >,
        hovered_sound: Res<ButtonHoveredSound>,
        pressed_sound: Res<ButtonPressedSound>,
    ) {
        for (interaction, mut color, selected) in &mut interaction_query {
            if *interaction == Interaction::Hovered {
                commands.spawn(AudioBundle {
                    source: hovered_sound.0.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            if *interaction == Interaction::Pressed {
                commands.spawn(AudioBundle {
                    source: pressed_sound.0.clone(),
                    settings: PlaybackSettings::DESPAWN,
                });
            }
            *color = match (*interaction, selected) {
                (Interaction::Pressed, _) | (Interaction::None, Some(_)) => PRESSED_BUTTON.into(),
                (Interaction::Hovered, Some(_)) => HOVERED_PRESSED_BUTTON.into(),
                (Interaction::Hovered, None) => HOVERED_BUTTON.into(),
                (Interaction::None, None) => NORMAL_BUTTON.into(),
            }
        }
    }

    fn menu_action(
        interaction_query: Query<
            (&Interaction, &MenuButtonAction),
            (Changed<Interaction>, With<Button>),
        >,
        mut app_exit_events: EventWriter<AppExit>,
        mut game_state: ResMut<NextState<GameState>>,
    ) {
        for (interaction, menu_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match menu_button_action {
                    MenuButtonAction::Play => game_state.set(GameState::Game),
                    MenuButtonAction::Quit => app_exit_events.send(AppExit),
                }
            }
        }
    }
}

pub mod results {
    use super::{despawn_screen, GameState};
    use crate::{constants::RESULTS_TEXT_SIZE, logic::GameEvent};
    use bevy::prelude::*;

    pub struct ResultsPlugin;

    impl Plugin for ResultsPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(OnEnter(GameState::Results), results_setup)
                .add_systems(Update, countdown.run_if(in_state(GameState::Results)))
                .add_systems(
                    OnExit(GameState::Results),
                    despawn_screen::<OnResultsScreen>,
                );
        }
    }

    #[derive(Component)]
    struct OnResultsScreen;

    #[derive(Resource, Deref, DerefMut)]
    pub struct ResultsTimer(pub Timer);

    fn results_setup(
        mut commands: Commands,
        mut events: EventReader<GameEvent>,
        asset_server: Res<AssetServer>,
    ) {
        // should not be empty
        assert!(!events.is_empty());

        let event = events.read().last().unwrap();
        let (results, text, color) = match event {
            GameEvent::GameOver(why) => ("Game over!", why.clone(), Color::RED),
            GameEvent::GameWon => ("Good job!", "Congratulations!".to_string(), Color::GREEN),
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                },
                OnResultsScreen,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::BLACK.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        parent.spawn(
                            TextBundle::from_section(
                                results.to_uppercase(),
                                TextStyle {
                                    font_size: RESULTS_TEXT_SIZE,
                                    color,
                                    font: asset_server.load("font.ttf"),
                                },
                            )
                            .with_style(Style {
                                margin: UiRect::all(Val::Px(50.0)),
                                ..default()
                            }),
                        );
                    })
                    .with_children(|parent| {
                        parent.spawn(
                            TextBundle::from_section(
                                text.to_uppercase(),
                                TextStyle {
                                    font_size: RESULTS_TEXT_SIZE,
                                    color,
                                    font: asset_server.load("font.ttf"),
                                },
                            )
                            .with_style(Style {
                                margin: UiRect::all(Val::Px(50.0)),
                                ..default()
                            }),
                        );
                    });
            });
    }

    fn countdown(
        mut game_state: ResMut<NextState<GameState>>,
        time: Res<Time>,
        mut timer: ResMut<ResultsTimer>,
    ) {
        if timer.tick(time.delta()).finished() {
            game_state.set(GameState::Menu);
        }
    }
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

fn get_scoreboard_position() -> f32 {
    let window_height = get_window_resolution().1;
    let top_wall_height = WallLocation::position(&WallLocation::Top).y;
    window_height / 2.0 - top_wall_height - 10.0
}

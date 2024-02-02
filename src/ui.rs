#![allow(clippy::type_complexity)]

use bevy::prelude::*;

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
    use crate::constants::SPLASH_SCREEN_DURATION;
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
                        width: Val::Px(200.0),
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
    use super::GameState;
    use crate::constants::*;
    use crate::game_logic::*;
    use crate::*;
    use bevy::prelude::*;

    pub struct GamePlugin;

    impl Plugin for GamePlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(OnEnter(GameState::Game), game_setup)
                .add_systems(
                    FixedUpdate,
                    (
                        handle_input,
                        move_snake,
                        check_for_collisions,
                        update_scoreboard,
                        handle_events,
                    )
                        .chain()
                        .run_if(in_state(GameState::Game)),
                )
                .add_systems(
                    OnExit(GameState::Game),
                    (despawn_screen::<OnGameScreen>, reset_state),
                );
        }
    }

    #[derive(Component)]
    pub struct OnGameScreen;

    pub fn game_setup(mut commands: Commands) {
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

        // The scoreboard
        let padding = (WINDOW_PADDING - SCOREBOARD_FONT_SIZE) / 2.0;
        commands.spawn((
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
            OnGameScreen,
        ));
    }

    fn reset_state(
        mut scoreboard: ResMut<Scoreboard>,
        mut player_input: ResMut<PlayerInput>,
        mut snake_body: ResMut<SnakeBody>,
        mut timer: ResMut<GameTimer>,
    ) {
        scoreboard.value = 0;
        snake_body.clear();
        player_input.0 = None;
        timer.reset();
    }
}

pub mod menu {
    use super::results::ResultsTimer;
    use super::{despawn_screen, GameState};
    use crate::constants::*;
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

    pub fn menu_setup(mut commands: Commands, mut timer: ResMut<ResultsTimer>) {
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
            color: MENU_TEXT_COLOR,
            ..default()
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
                        background_color: Color::DARK_GREEN.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        // Game name
                        parent.spawn(
                            TextBundle::from_section(
                                "Snake",
                                TextStyle {
                                    font_size: MENU_TITLE_SIZE,
                                    color: MENU_TEXT_COLOR,
                                    ..default()
                                },
                            )
                            .with_style(Style {
                                margin: UiRect::all(Val::Px(50.0)),
                                ..default()
                            }),
                        );

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
                                    "Play",
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
                                parent.spawn(TextBundle::from_section("Quit", button_text_style));
                            });
                    });
            });

        timer.reset();
    }

    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor, Option<&SelectedOption>),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut color, selected) in &mut interaction_query {
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
    use crate::constants::{RESULTS_TEXT_COLOR, RESULTS_TEXT_SIZE};
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

    fn results_setup(mut commands: Commands) {
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
                        background_color: Color::DARK_GREEN.into(),
                        ..default()
                    })
                    .with_children(|parent| {
                        // Game name
                        parent.spawn(
                            TextBundle::from_section(
                                "Results",
                                TextStyle {
                                    font_size: RESULTS_TEXT_SIZE,
                                    color: RESULTS_TEXT_COLOR,
                                    ..default()
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

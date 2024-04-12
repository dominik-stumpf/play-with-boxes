use crate::loading::TextureAssets;
use crate::GameState;
use bevy::prelude::*;
use bevy_simple_text_input::{
    TextInputBundle, TextInputPlugin, TextInputSubmitEvent, TextInputValue,
};

pub struct MenuPlugin;

/// This plugin is responsible for the game menu (containing only one button...)
/// The menu is only drawn during the State `GameState::Menu` and is removed when that state is exited
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), setup_menu)
            .add_systems(
                Update,
                (button_system, button_style_system).run_if(in_state(GameState::Menu)),
            )
            .add_plugins(TextInputPlugin)
            .add_systems(OnExit(GameState::Menu), cleanup_menu);
    }
}

const BACKGROUND_DIM: Color = Color::rgb(0.2, 0.2, 0.2);
const BACKGROUND: Color = Color::rgb(0.0, 0.0, 0.0);
const FOREGROUND_DIM: Color = Color::rgb(0.5, 0.5, 0.5);
const FOREGROUND: Color = Color::rgb(0.9, 0.9, 0.9);

#[derive(Component)]
struct Menu;

#[derive(Component)]
struct JoinButton;

fn setup_menu(mut commands: Commands, textures: Res<TextureAssets>) {
    info!("menu");
    commands.spawn(Camera2dBundle::default());

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    top: Val::Px(8.),
                    left: Val::Px(8.),
                    width: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    ..default()
                },
                ..default()
            },
            Menu,
        ))
        .with_children(|children| {
            children.spawn(TextBundle::from_section(
                "PLAY WITH BOXES",
                TextStyle {
                    font_size: 256.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ));
        });

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    row_gap: Val::Px(16.0),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            Menu,
        ))
        .with_children(|node| {
            // address input field
            node.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Px(300.0),
                        border: UiRect::all(Val::Px(1.0)),
                        padding: UiRect::all(Val::Px(4.0)),
                        ..default()
                    },
                    background_color: BACKGROUND.into(),
                    ..default()
                },
                TextInputBundle::default().with_text_style(TextStyle {
                    font_size: 24.,
                    color: FOREGROUND,
                    ..default()
                }),
            ));

            // join button
            node.spawn((
                ButtonBundle {
                    style: Style {
                        margin: UiRect {
                            bottom: Val::Px(12.0),
                            ..default()
                        },
                        width: Val::Px(300.0),
                        height: Val::Px(32.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    background_color: BACKGROUND.into(),
                    ..Default::default()
                },
                JoinButton,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "JOIN",
                    TextStyle {
                        font_size: 24.0,
                        color: FOREGROUND,
                        ..default()
                    },
                ));
            });

            // host button
            node.spawn((ButtonBundle {
                style: Style {
                    width: Val::Px(300.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                background_color: BACKGROUND.into(),
                ..Default::default()
            },))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "HOST",
                        TextStyle {
                            font_size: 24.0,
                            color: FOREGROUND,
                            ..default()
                        },
                    ));
                });
        });
}

fn button_system(
    interaction_query: Query<&Interaction, (Changed<Interaction>, With<JoinButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
    text_input_query: Query<&TextInputValue>,
) {
    for interaction in &interaction_query {
        if !matches!(interaction, Interaction::Pressed) {
            continue;
        }

        let text_input = text_input_query.single();
        let current_value = text_input.0.parse::<String>().unwrap_or("".to_string());
        println!("{current_value}");
        next_state.set(GameState::Playing);
    }
}

fn button_style_system(
    mut interaction_query: Query<
        (&Interaction, &mut BorderColor, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut border_color, mut background_color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *background_color = BACKGROUND_DIM.into();
            }
            Interaction::Hovered => {
                *background_color = FOREGROUND_DIM.into();
            }
            Interaction::None => {
                *background_color = BACKGROUND.into();
            }
        }
    }
}

fn cleanup_menu(mut commands: Commands, menu: Query<Entity, With<Menu>>) {
    for entity in menu.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

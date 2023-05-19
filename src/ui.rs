use bevy::prelude::*;

use crate::local_chalk::LocalChalk;

pub(crate) struct UiPlugin;

pub(crate) fn default_font(asset_server: &Res<AssetServer>) -> Handle<Font> {
    asset_server.load("fonts/FiraMono-Medium.ttf")
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(setup)
            .add_system(color_btn_system)
            .add_system(incr_btn_system)
            .add_system(decr_btn_system);
    }
}

// Add a green button in the bottom left corner
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, chalk: ResMut<LocalChalk>) {
    let font = default_font(&asset_server);
    const FONT_SIZE: f32 = 40.0;
    const BTN_WIDTH: f32 = 50.0;

    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(0.),
                    bottom: Val::Px(0.),
                    ..default()
                },
                size: Size::new(Val::Percent(100.), Val::Px(50.)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(NodeBundle {
                    style: Style {
                        position: UiRect {
                            left: Val::Px(0.),
                            bottom: Val::Px(0.),
                            ..default()
                        },
                        ..default()
                    },
                    background_color: BackgroundColor(Color::BLUE),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn((
                        ColorButton,
                        ButtonBundle {
                            style: Style {
                                size: Size::new(Val::Px(BTN_WIDTH), Val::Percent(100.)),
                                ..default()
                            },
                            background_color: BackgroundColor(chalk.get().color),
                            ..default()
                        },
                    ));
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            DecrementButton,
                            ButtonBundle {
                                style: Style {
                                    size: Size::width(Val::Px(BTN_WIDTH)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::DARK_GRAY),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "-",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: FONT_SIZE,
                                    color: Color::WHITE,
                                },
                            ));
                        });
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            IncrementButton,
                            ButtonBundle {
                                style: Style {
                                    size: Size::width(Val::Px(BTN_WIDTH)),
                                    justify_content: JustifyContent::Center,
                                    align_items: AlignItems::Center,
                                    ..default()
                                },
                                background_color: BackgroundColor(Color::DARK_GRAY),
                                ..default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                "+",
                                TextStyle {
                                    font: font.clone(),
                                    font_size: FONT_SIZE,
                                    color: Color::WHITE,
                                },
                            ));
                        });
                });
        });
}

#[derive(Component)]
struct ColorButton;

#[derive(Component)]
struct IncrementButton;

#[derive(Component)]
struct DecrementButton;

fn color_btn_system(
    mut chalk: ResMut<LocalChalk>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<ColorButton>),
    >,
) {
    for (interaction, mut bg) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *bg = chalk.as_mut().next_color().into();
            }
            _ => {}
        }
    }
}

fn incr_btn_system(
    mut chalk: ResMut<LocalChalk>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<IncrementButton>)>,
) {
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                chalk.as_mut().incr_size();
            }
            _ => {}
        }
    }
}

fn decr_btn_system(
    mut chalk: ResMut<LocalChalk>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<DecrementButton>)>,
) {
    for interaction in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                chalk.as_mut().decr_size();
            }
            _ => {}
        }
    }
}

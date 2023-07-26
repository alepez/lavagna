#![allow(clippy::type_complexity)]
#![allow(clippy::needless_pass_by_value)]

use bevy::prelude::*;

use crate::{drawing::ClearEvent, local_chalk::LocalChalk, Stats};

#[derive(Copy, Clone, Resource)]
struct UiPluginOpt {
    visible: bool,
}

pub(crate) struct UiPlugin(UiPluginOpt);

impl UiPlugin {
    pub fn new(visible: bool) -> Self {
        Self(UiPluginOpt { visible })
    }
}

pub(crate) fn default_font(asset_server: &Res<AssetServer>) -> Handle<Font> {
    asset_server.load("fonts/FiraMono-Medium.ttf")
}

impl Plugin for UiPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.0)
            .add_systems(Startup, setup)
            .add_event::<ToggleUiEvent>()
            .add_systems(Update, toggle_ui_system)
            .add_systems(Update, color_btn_system)
            .add_systems(Update, clear_btn_system)
            .add_systems(Update, incr_btn_system)
            .add_systems(Update, update_collab_info)
            .add_systems(Update, decr_btn_system);
    }
}

#[derive(Component)]
struct Toolbar;

// Add a green button in the bottom left corner
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, opt: Res<UiPluginOpt>) {
    let font = default_font(&asset_server);

    let visibility = if opt.visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    commands
        .spawn((
            Toolbar,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.),
                    bottom: Val::Px(0.),
                    width: Val::Percent(100.),
                    height: Val::Px(50.),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                visibility,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((ColorButton, button()));
        })
        .with_children(|parent| {
            parent
                .spawn((DecrementButton, button()))
                .with_children(|parent| {
                    parent.spawn(label("-", &font));
                });
        })
        .with_children(|parent| {
            parent
                .spawn((IncrementButton, button()))
                .with_children(|parent| {
                    parent.spawn(label("+", &font));
                });
        })
        .with_children(|parent| {
            parent
                .spawn((ClearButton, button()))
                .with_children(|parent| {
                    parent.spawn(label("x", &font));
                });
        })
        .with_children(|parent| {
            parent.spawn(button()).with_children(|parent| {
                parent.spawn((CollabText, label(" ", &font)));
            });
        });
}

fn label(text: &str, font: &Handle<Font>) -> TextBundle {
    const FONT_SIZE: f32 = 40.0;

    TextBundle {
        text: Text::from_section(
            text,
            TextStyle {
                font: font.clone(),
                font_size: FONT_SIZE,
                color: Color::WHITE,
            },
        ),
        ..default()
    }
}

fn button() -> ButtonBundle {
    const BTN_WIDTH: f32 = 50.0;

    ButtonBundle {
        style: Style {
            width: Val::Px(BTN_WIDTH),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        background_color: BackgroundColor(Color::Rgba {
            red: 0.2,
            green: 0.2,
            blue: 0.2,
            alpha: 0.75,
        }),
        ..default()
    }
}

#[derive(Component)]
struct ColorButton;

#[derive(Component)]
struct IncrementButton;

#[derive(Component)]
struct DecrementButton;

#[derive(Component)]
struct ClearButton;

#[derive(Component)]
struct CollabText;

fn color_btn_system(
    mut chalk: ResMut<LocalChalk>,
    mut btn_query: Query<&mut BackgroundColor, With<ColorButton>>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<ColorButton>)>,
) {
    let mut bg = btn_query.single_mut();
    *bg = chalk.as_mut().color().into();
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            *bg = chalk.as_mut().next_color().into();
        }
    }
}

fn incr_btn_system(
    mut chalk: ResMut<LocalChalk>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<IncrementButton>)>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            chalk.as_mut().grow();
        }
    }
}

fn decr_btn_system(
    mut chalk: ResMut<LocalChalk>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<DecrementButton>)>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            chalk.as_mut().shrink();
        }
    }
}

fn clear_btn_system(
    mut event: EventWriter<ClearEvent>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<ClearButton>)>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Pressed {
            event.send(ClearEvent::new());
        }
    }
}

#[derive(Event)]
pub(crate) struct ToggleUiEvent;

fn toggle_ui_system(
    mut events: EventReader<ToggleUiEvent>,
    mut visibility_q: Query<&mut Visibility, With<Toolbar>>,
) {
    let visibility: &mut Visibility = &mut visibility_q.single_mut();

    for _ in events.iter() {
        *visibility = match *visibility {
            Visibility::Visible => Visibility::Hidden,
            Visibility::Hidden => Visibility::Visible,
            x @ Visibility::Inherited => x,
        };
    }
}

fn update_collab_info(mut txt_query: Query<&mut Text, With<CollabText>>, stats: Res<Stats>) {
    let text = if stats.collab.active {
        if stats.collab.peers == 0 {
            "…".to_string()
        } else {
            format!("{}", stats.collab.peers)
        }
    } else {
        String::new()
    };

    txt_query.single_mut().sections[0].value = text;
}

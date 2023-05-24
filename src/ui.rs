#![allow(clippy::type_complexity)]

use bevy::prelude::*;

use crate::{drawing::ClearEvent, local_chalk::LocalChalk};

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
            .add_startup_system(setup)
            .add_event::<ToggleUiEvent>()
            .add_system(toggle_ui_system)
            .add_system(color_btn_system)
            .add_system(clear_btn_system)
            .add_system(incr_btn_system)
            .add_system(decr_btn_system);
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
                    position: UiRect {
                        left: Val::Px(0.),
                        bottom: Val::Px(0.),
                        ..default()
                    },
                    size: Size::new(Val::Percent(100.), Val::Px(50.)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                visibility,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((ColorButton, label()));
        })
        .with_children(|parent| {
            parent
                .spawn((DecrementButton, label()))
                .with_children(|parent| {
                    parent.spawn(button("-", &font));
                });
        })
        .with_children(|parent| {
            parent
                .spawn((IncrementButton, label()))
                .with_children(|parent| {
                    parent.spawn(button("+", &font));
                });
        })
        .with_children(|parent| {
            parent
                .spawn((ClearButton, label()))
                .with_children(|parent| {
                    parent.spawn(button("x", &font));
                });
        });
}

fn button(text: &str, font: &Handle<Font>) -> TextBundle {
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

fn label() -> ButtonBundle {
    const BTN_WIDTH: f32 = 50.0;

    ButtonBundle {
        style: Style {
            size: Size::width(Val::Px(BTN_WIDTH)),
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

fn color_btn_system(
    mut chalk: ResMut<LocalChalk>,
    mut btn_query: Query<&mut BackgroundColor, With<ColorButton>>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<ColorButton>)>,
) {
    let mut bg = btn_query.single_mut();
    *bg = chalk.as_mut().color().into();
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Clicked {
            *bg = chalk.as_mut().next_color().into();
        }
    }
}

fn incr_btn_system(
    mut chalk: ResMut<LocalChalk>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<IncrementButton>)>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Clicked {
            chalk.as_mut().grow();
        }
    }
}

fn decr_btn_system(
    mut chalk: ResMut<LocalChalk>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<DecrementButton>)>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Clicked {
            chalk.as_mut().shrink();
        }
    }
}

fn clear_btn_system(
    mut event: EventWriter<ClearEvent>,
    mut interaction_query: Query<&Interaction, (Changed<Interaction>, With<ClearButton>)>,
) {
    for interaction in &mut interaction_query {
        if *interaction == Interaction::Clicked {
            event.send(ClearEvent::new());
        }
    }
}

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
            x => x,
        };
    }
}

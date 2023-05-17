use crate::local_chalk::LocalChalkConfig;
use bevy::prelude::*;

pub(crate) struct KeybindingPlugin;

impl Plugin for KeybindingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(update);
    }
}

const COLORS: [Color; 7] = [
    Color::WHITE,
    Color::BLUE,
    Color::TURQUOISE,
    Color::GREEN,
    Color::YELLOW,
    Color::ORANGE,
    Color::RED,
];

fn next_color(curr_color: Color) -> Color {
    if let Some(next_color) = COLORS
        .iter()
        .cycle()
        .skip_while(|&&x| x != curr_color)
        .skip(1)
        .next()
    {
        *next_color
    } else {
        curr_color
    }
}

fn update(keyboard_input: Res<Input<KeyCode>>, mut chalk_config: ResMut<LocalChalkConfig>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        println!("TODO Quit the application");
    }

    if keyboard_input.just_pressed(KeyCode::X) {
        println!("TODO Take a snapshot and clear everything");
    }

    if keyboard_input.just_pressed(KeyCode::C) {
        println!("TODO Change the chalk color");
        chalk_config.color = next_color(chalk_config.color);
    }

    if keyboard_input.just_pressed(KeyCode::U) {
        println!("TODO Resume the last snapshot");
    }

    if keyboard_input.just_pressed(KeyCode::S) {
        println!("TODO Take a snapshot");
    }

    if keyboard_input.just_pressed(KeyCode::M) {
        if chalk_config.line_width < 100 {
            chalk_config.line_width = chalk_config.line_width * 2;
        }
    }

    if keyboard_input.just_pressed(KeyCode::N) {
        if chalk_config.line_width > 1 {
            chalk_config.line_width = chalk_config.line_width / 2;
        }
    }
}

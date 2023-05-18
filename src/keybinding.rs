use crate::local_chalk::LocalChalk;
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
        .nth(1)
    {
        *next_color
    } else {
        curr_color
    }
}

fn update(keyboard_input: Res<Input<KeyCode>>, mut chalk: ResMut<LocalChalk>) {
    let chalk = &mut chalk.get_mut();

    if keyboard_input.just_pressed(KeyCode::Escape) {
        println!("TODO Quit the application");
    }

    if keyboard_input.just_pressed(KeyCode::X) {
        println!("TODO Take a snapshot and clear everything");
    }

    if keyboard_input.just_pressed(KeyCode::C) {
        println!("TODO Change the chalk color");
        chalk.color = next_color(chalk.color);
    }

    if keyboard_input.just_pressed(KeyCode::U) {
        println!("TODO Resume the last snapshot");
    }

    if keyboard_input.just_pressed(KeyCode::S) {
        println!("TODO Take a snapshot");
    }

    if keyboard_input.just_pressed(KeyCode::M) && chalk.line_width < 100 {
        chalk.line_width *= 2;
    }

    if keyboard_input.just_pressed(KeyCode::N) && chalk.line_width > 1 {
        chalk.line_width /= 2;
    }
}

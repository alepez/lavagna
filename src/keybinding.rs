use crate::local_chalk::LocalChalk;
use bevy::prelude::*;

pub(crate) struct KeybindingPlugin;

impl Plugin for KeybindingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(update);
    }
}

fn update(keyboard_input: Res<Input<KeyCode>>, mut chalk: ResMut<LocalChalk>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        println!("TODO Quit the application");
    }

    if keyboard_input.just_pressed(KeyCode::X) {
        println!("TODO Take a snapshot and clear everything");
    }

    if keyboard_input.just_pressed(KeyCode::C) {
        println!("TODO Change the chalk color");
        chalk.next_color();
    }

    if keyboard_input.just_pressed(KeyCode::U) {
        println!("TODO Resume the last snapshot");
    }

    if keyboard_input.just_pressed(KeyCode::S) {
        println!("TODO Take a snapshot");
    }

    if keyboard_input.just_pressed(KeyCode::M) {
        chalk.incr_size();
    }

    if keyboard_input.just_pressed(KeyCode::N) {
        chalk.decr_size();
    }
}

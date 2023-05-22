use crate::local_chalk::LocalChalk;
use bevy::prelude::*;

pub(crate) struct KeybindingPlugin;

impl Plugin for KeybindingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(update);
    }
}

fn update(keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        println!("TODO Quit the application");
    }

    if keyboard_input.just_pressed(KeyCode::X) {
        println!("TODO Take a snapshot and clear everything");
    }

    if keyboard_input.just_pressed(KeyCode::U) {
        println!("TODO Resume the last snapshot");
    }

    if keyboard_input.just_pressed(KeyCode::S) {
        println!("TODO Take a snapshot");
    }
}

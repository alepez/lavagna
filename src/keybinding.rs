use crate::drawing::ClearEvent;
use bevy::prelude::*;

pub(crate) struct KeybindingPlugin;

impl Plugin for KeybindingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(update);
    }
}

fn update(keyboard_input: Res<Input<KeyCode>>, mut event: EventWriter<ClearEvent>) {
    if keyboard_input.just_pressed(KeyCode::X) {
        event.send(ClearEvent);
    }

    if keyboard_input.just_pressed(KeyCode::C) {
        // chalk.next_color();
    }

    if keyboard_input.just_pressed(KeyCode::M) {
        // chalk.incr_size();
    }

    if keyboard_input.just_pressed(KeyCode::N) {
        // chalk.decr_size();
    }
}

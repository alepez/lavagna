use crate::{
    drawing::ClearEvent,
    local_chalk::{ChangeColorEvent, GrowEvent, ShrinkEvent},
};
use bevy::prelude::*;

pub(crate) struct KeybindingPlugin;

impl Plugin for KeybindingPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_system(update);
    }
}

fn update(
    keyboard_input: Res<Input<KeyCode>>,
    mut clear_event: EventWriter<ClearEvent>,
    mut change_color_event: EventWriter<ChangeColorEvent>,
    mut shrink_event: EventWriter<ShrinkEvent>,
    mut grow_event: EventWriter<GrowEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::X) {
        clear_event.send(ClearEvent);
    }

    if keyboard_input.just_pressed(KeyCode::C) {
        change_color_event.send(ChangeColorEvent);
    }

    if keyboard_input.just_pressed(KeyCode::M) {
        grow_event.send(GrowEvent);
    }

    if keyboard_input.just_pressed(KeyCode::N) {
        shrink_event.send(ShrinkEvent);
    }
}

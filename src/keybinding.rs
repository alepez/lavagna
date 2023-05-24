use crate::{
    drawing::ClearEvent,
    local_chalk::{ChangeColorEvent, GrowEvent, ShrinkEvent},
    ui::ToggleUiEvent,
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
    mut toggle_ui_event: EventWriter<ToggleUiEvent>,
) {
    if keyboard_input.just_pressed(KeyCode::X) {
        clear_event.send(ClearEvent::new());
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

    if keyboard_input.just_pressed(KeyCode::U) {
        toggle_ui_event.send(ToggleUiEvent);
    }
}

#[derive(Default)]
pub struct App {
    pub input: InputState,
}

#[derive(Default)]
pub struct InputState {
    pub pressed: bool,
}

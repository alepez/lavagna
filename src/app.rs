use log::debug;

#[derive(Debug)]
pub struct App {
    input: InputState,
    canvas: Canvas,
}

pub struct AppBuilder {
    app: App,
}

impl AppBuilder {
    pub fn new() -> AppBuilder {
        AppBuilder {
            app: App {
                input: Default::default(),
                canvas: Default::default(),
            }
        }
    }

    pub fn with_size(mut self, width: u32, height: u32) -> AppBuilder {
        self.app.canvas.width = width;
        self.app.canvas.height = height;
        self
    }

    pub fn build(self) -> App {
        self.app
    }
}


#[derive(Default, Debug)]
pub struct Canvas {
    width: u32,
    height: u32,
}

#[derive(Default, Debug)]
pub struct InputState {
    pressed: bool,
    pos: CursorPos,
}

#[derive(Default, Debug)]
pub struct CursorPos {
    x: u32,
    y: u32,
}

impl App {
    pub fn update(&mut self, frame: &mut [u8]) {
        if self.input.pressed {
            self.draw_with_brush(frame);
        }
    }

    pub fn set_position(&mut self, x: u32, y: u32) {
        self.input.pos.x = x;
        self.input.pos.y = y;
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.input.pressed = pressed;
    }

    fn draw_with_brush(&mut self, frame: &mut [u8]) {
        let CursorPos { x, y } = self.input.pos;
        let pix_index = (self.canvas.width * y + x) as usize;

        debug!("Mouse pressed at {:?}", self.input.pos);

        if let Some(pix) = frame
            .chunks_exact_mut(4)
            .skip(pix_index).next() {
            let color = [0, 0xff, 0xff, 0xff];
            pix.copy_from_slice(&color);
        }
    }
}
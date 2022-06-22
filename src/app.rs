use log::debug;

#[derive(Debug)]
pub struct App {
    pub input: InputState,
    canvas: Canvas,
}

pub struct AppBuilder {
    app: App,
}

impl AppBuilder {
    pub fn build() -> AppBuilder {
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

    pub fn app(self) -> App {
        self.app
    }
}


#[derive(Default, Debug)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
}

#[derive(Default, Debug)]
pub struct InputState {
    pub pressed: bool,
    pub pos: CursorPos,
}

#[derive(Default, Debug)]
pub struct CursorPos {
    pub x: u32,
    pub y: u32,
}

impl App {
    pub fn update(&mut self, frame: &mut [u8]) {
        if self.input.pressed {
            self.draw_with_brush(frame);
        }
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
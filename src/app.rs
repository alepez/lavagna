use log::debug;

#[derive(Debug)]
pub struct App {
    input: Cursor,
    canvas: Canvas,
}

#[derive(Default)]
pub struct AppBuilder {
    width: u32,
    height: u32,
}

impl AppBuilder {
    pub fn new() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn with_size(mut self, width: u32, height: u32) -> AppBuilder {
        self.width = width;
        self.height = height;
        self
    }

    pub fn build(self) -> App {
        let AppBuilder { width, height } = self;

        App {
            input: Cursor { pressed: false, pos: CursorPos { x: 0, y: 0 } },
            canvas: Canvas { width, height },
        }
    }
}


#[derive(Default, Debug)]
struct Canvas {
    width: u32,
    #[allow(dead_code)]
    height: u32,
}

#[derive(Default, Debug)]
struct Cursor {
    pressed: bool,
    pos: CursorPos,
}

#[derive(Default, Debug)]
struct CursorPos {
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
            let color = [0xff, 0xff, 0xff, 0xff];
            pix.copy_from_slice(&color);
        }
    }
}
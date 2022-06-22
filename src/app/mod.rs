mod line;

use crate::app::line::BresenhamLine;

#[derive(Debug)]
pub struct App {
    cursor: Cursor,
    prev_cursor: Cursor,
    canvas: Canvas,
}

#[derive(Default)]
pub struct AppBuilder {
    width: isize,
    height: isize,
}

impl AppBuilder {
    pub fn new() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn with_size(mut self, width: isize, height: isize) -> AppBuilder {
        self.width = width;
        self.height = height;
        self
    }

    pub fn build(self) -> App {
        let AppBuilder { width, height } = self;

        App {
            cursor: Cursor::default(),
            prev_cursor: Cursor::default(),
            canvas: Canvas { width, height },
        }
    }
}


#[derive(Default, Debug)]
struct Canvas {
    width: isize,
    #[allow(dead_code)]
    height: isize,
}

#[derive(Default, Debug, Copy, Clone)]
struct Cursor {
    pressed: bool,
    pos: CursorPos,
}

#[derive(Default, Debug, Copy, Clone)]
pub struct CursorPos {
    x: isize,
    y: isize,
}

impl App {
    pub fn update(&mut self, frame: &mut [u8]) {
        let mut painter = Painter { frame, width: self.canvas.width };

        if self.cursor.pressed {
            if self.prev_cursor.pressed {
                painter.draw_line(self.prev_cursor.pos, self.cursor.pos);
            }

            painter.draw_pixel(self.cursor.pos);
        }

        self.prev_cursor = self.cursor;
    }

    pub fn set_position(&mut self, x: isize, y: isize) {
        self.cursor.pos.x = x;
        self.cursor.pos.y = y;
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.cursor.pressed = pressed;
    }
}

struct Painter<'a> {
    frame: &'a mut [u8],
    width: isize,
}

impl<'a> Painter<'a> {
    fn draw_line(&mut self, from: CursorPos, to: CursorPos) {
        BresenhamLine::new(from, to).for_each(|p| {
            self.draw_pixel(p);
        });
    }

    fn draw_pixel(&mut self, pos: CursorPos) {
        let CursorPos { x, y } = pos;

        let pix_index = (self.width * y + x) as usize;

        if let Some(pix) = self.frame
            .chunks_exact_mut(4)
            .skip(pix_index).next() {
            let color = [0xff, 0xff, 0xff, 0xff];
            pix.copy_from_slice(&color);
        }
    }
}
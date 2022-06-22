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
struct CursorPos {
    x: isize,
    y: isize,
}

impl App {
    pub fn update(&mut self, frame: &mut [u8]) {
        if self.cursor.pressed {
            if self.prev_cursor.pressed {
                self.draw_line(frame, self.prev_cursor.pos, self.cursor.pos);
            }

            self.draw_pixel(frame, self.cursor.pos);
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

    fn draw_line(&mut self, frame: &mut [u8], from: CursorPos, to: CursorPos) {
        BresenhamLine::new(from, to).for_each(|p| {
            self.draw_pixel(frame, p);
        });
    }

    fn draw_pixel(&mut self, frame: &mut [u8], pos: CursorPos) {
        let CursorPos { x, y } = pos;

        let pix_index = (self.canvas.width * y + x) as usize;

        if let Some(pix) = frame
            .chunks_exact_mut(4)
            .skip(pix_index).next() {
            let color = [0xff, 0xff, 0xff, 0xff];
            pix.copy_from_slice(&color);
        }
    }
}

struct BresenhamLine {
    error: isize,
    x: isize,
    y: isize,
    sx: isize,
    sy: isize,
    dx: isize,
    dy: isize,
    to: CursorPos,
}

impl BresenhamLine {
    fn new(from: CursorPos, to: CursorPos) -> BresenhamLine {
        let dx = (to.x - from.x).abs();
        let dy = (to.y - from.y).abs();

        BresenhamLine {
            error: (if dx > dy { dx } else { -dy }) / 2,
            x: from.x,
            y: from.y,
            sx: if from.x < to.x { 1 } else { -1 },
            sy: if from.y < to.y { 1 } else { -1 },
            dx,
            dy,
            to,
        }
    }
}

impl Iterator for BresenhamLine {
    type Item = CursorPos;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.x != self.to.x) || (self.y != self.to.y) {
            let next = CursorPos { x: self.x, y: self.y };

            let e = self.error;

            if e > -self.dx {
                self.error -= self.dy;
                self.x += self.sx;
            }

            if e < self.dy {
                self.error += self.dx;
                self.y += self.sy;
            }

            Some(next)
        } else {
            None
        }
    }
}
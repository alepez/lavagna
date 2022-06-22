mod line;
mod painter;
mod color;

use std::collections::VecDeque;
use crate::app::color::*;
use crate::app::painter::Painter;

#[derive(Debug)]
enum Command {
    ClearAll,
}

pub struct App {
    cursor: Cursor,
    prev_cursor: Cursor,
    canvas: Canvas,
    commands: VecDeque<Command>,
    palette: ColorSelector,
    color: Color,
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
        let mut palette = ColorSelector::new(&PALETTE);
        let color = palette.next().unwrap();

        App {
            cursor: Cursor::default(),
            prev_cursor: Cursor::default(),
            canvas: Canvas { width, height },
            commands: VecDeque::with_capacity(10),
            palette,
            color,
        }
    }
}


#[derive(Default, Debug)]
pub struct Canvas {
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
        let mut painter = Painter::new(frame, &self.canvas, self.color);

        while let Some(command) = self.commands.pop_front() {
            match command {
                Command::ClearAll => { painter.clear(); }
            }
        }

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

    pub fn resize(&mut self, width: isize, height: isize) {
        self.canvas.width = width;
        self.canvas.height = height;
    }

    pub fn clear_all(&mut self) {
        self.commands.push_back(Command::ClearAll);
    }

    pub fn change_color(&mut self) {
        if let Some(color) = self.palette.next() {
            self.color = color;
        }
    }
}

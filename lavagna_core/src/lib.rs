mod color;
pub mod doc;
mod painter;

use crate::color::*;
use crate::doc::{MutSketch, OwnedSketch};
use crate::painter::Painter;
use std::collections::VecDeque;

#[derive(Debug)]
enum Command {
    ClearAll,
    Resume,
    TakeSnapshot,
}

pub struct App {
    cursor: Cursor,
    prev_cursor: Cursor,
    commands: VecDeque<Command>,
    palette: ColorSelector,
    color: Color,
    snapshots: Vec<OwnedSketch>,
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
    pub fn new() -> Self {
        let mut palette = ColorSelector::new(&PALETTE);
        let color = palette.next().unwrap();

        App {
            cursor: Cursor::default(),
            prev_cursor: Cursor::default(),
            commands: VecDeque::with_capacity(10),
            palette,
            color,
            snapshots: Vec::new(),
        }
    }

    pub fn update(&mut self, mut sketch: MutSketch) {
        while let Some(command) = self.commands.pop_front() {
            match command {
                Command::ClearAll => {
                    self.snapshots.push(sketch.to_owned());
                    sketch.frame.fill(0x00);
                }
                Command::TakeSnapshot => {
                    self.snapshots.push(sketch.to_owned());
                }
                Command::Resume => {
                    if let Some(backup) = &self.snapshots.pop() {
                        sketch.copy_from(&backup.as_sketch());
                    }
                }
            }
        }

        let mut painter = Painter::new(sketch, self.color);

        if self.cursor.pressed && self.prev_cursor.pressed {
            painter.draw_line(self.prev_cursor.pos, self.cursor.pos);
        } else {
            draw_current_color_icon(&mut painter);
        }

        self.prev_cursor = self.cursor;
    }

    pub fn set_cursor_position(&mut self, x: isize, y: isize) {
        self.cursor.pos.x = x;
        self.cursor.pos.y = y;
    }

    pub fn set_pressed(&mut self, pressed: bool) {
        self.cursor.pressed = pressed;
    }

    pub fn clear_all(&mut self) {
        self.commands.push_back(Command::ClearAll);
    }

    pub fn resume(&mut self) {
        self.commands.push_back(Command::Resume);
    }

    pub fn take_snapshot(&mut self) {
        self.commands.push_back(Command::TakeSnapshot);
    }

    pub fn change_color(&mut self) {
        if let Some(color) = self.palette.next() {
            self.color = color;
        }
    }
}

fn draw_current_color_icon(painter: &mut Painter) {
    const SQUARE_SIZE: isize = 10;

    for x in 0..SQUARE_SIZE {
        for y in 0..(SQUARE_SIZE - x) {
            painter.draw_pixel(CursorPos { x, y });
        }
    }
}

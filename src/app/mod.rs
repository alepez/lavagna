mod painter;
mod color;
pub(crate) mod doc;

use std::collections::VecDeque;
use crate::app::color::*;
use crate::app::doc::{MutSketch, OwnedSketch};
use crate::app::painter::Painter;

#[derive(Debug)]
enum Command {
    ClearAll,
    Resume,
    Backup,
}

pub struct App {
    cursor: Cursor,
    prev_cursor: Cursor,
    commands: VecDeque<Command>,
    palette: ColorSelector,
    color: Color,
    backups: Vec<OwnedSketch>,
}

#[derive(Default)]
pub struct AppBuilder {}

impl AppBuilder {
    pub fn new() -> AppBuilder {
        AppBuilder::default()
    }

    pub fn build(self) -> App {
        let mut palette = ColorSelector::new(&PALETTE);
        let color = palette.next().unwrap();

        App {
            cursor: Cursor::default(),
            prev_cursor: Cursor::default(),
            commands: VecDeque::with_capacity(10),
            palette,
            color,
            backups: Vec::new(),
        }
    }
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
    pub fn update(&mut self, mut sketch: MutSketch) {
        while let Some(command) = self.commands.pop_front() {
            match command {
                Command::ClearAll => {
                    self.backups.push(sketch.to_owned());
                    sketch.frame.fill(0x00);
                }
                Command::Backup => {
                    self.backups.push(sketch.to_owned());
                }
                Command::Resume => {
                    if let Some(backup) = &self.backups.pop() {
                        sketch.copy_from(&backup.as_sketch());
                    }
                }
            }
        }

        let mut painter = Painter::new(sketch, self.color);

        if self.cursor.pressed && self.prev_cursor.pressed {
            painter.draw_line(self.prev_cursor.pos, self.cursor.pos);
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

    pub fn clear_all(&mut self) {
        self.commands.push_back(Command::ClearAll);
    }

    pub fn resume(&mut self) {
        self.commands.push_back(Command::Resume);
    }

    pub fn backup(&mut self) {
        self.commands.push_back(Command::Backup);
    }

    pub fn change_color(&mut self) {
        if let Some(color) = self.palette.next() {
            self.color = color;
        }
    }
}

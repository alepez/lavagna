mod color;
pub mod doc;
mod painter;

use crate::color::*;
use crate::doc::{MutSketch, OwnedSketch};
use crate::painter::Painter;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Copy, Clone, Debug)]
pub struct CollabId(u32);

impl From<u32> for CollabId {
    fn from(x: u32) -> Self {
        Self(x)
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Command {
    ClearAll,
    Resume,
    TakeSnapshot,
    ChangeColor(Color),
    MoveCursor(CursorPos),
    Pressed,
    Released,
}

pub trait CommandSender {
    fn send_command(&mut self, cmd: Command);
}

pub struct App {
    cursor: Cursor,
    prev_cursor: Cursor,
    commands: VecDeque<Command>,
    palette: ColorSelector,
    color: Color,
    snapshots: Vec<OwnedSketch>,
    chained_command_sender: Option<Box<dyn FnMut(Command)>>,
}

#[derive(Default, Debug, Copy, Clone)]
struct Cursor {
    pressed: bool,
    pos: CursorPos,
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CursorPos {
    pub x: isize,
    pub y: isize,
}

impl Default for App {
    fn default() -> Self {
        let mut palette = ColorSelector::new(&PALETTE);
        let color = palette.next().unwrap();

        App {
            cursor: Cursor::default(),
            prev_cursor: Cursor::default(),
            commands: VecDeque::with_capacity(10),
            palette,
            color,
            snapshots: Vec::new(),
            chained_command_sender: Default::default(),
        }
    }
}

impl App {
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
                Command::ChangeColor(color) => {
                    self.color = color;
                }
                Command::MoveCursor(pos) => {
                    self.cursor.pos = pos;
                }
                Command::Pressed => {
                    self.cursor.pressed = true;
                }
                Command::Released => {
                    self.cursor.pressed = false;
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

    pub fn clear_all(&mut self) {
        self.send_command_chained(Command::ClearAll);
    }

    pub fn resume(&mut self) {
        self.send_command_chained(Command::Resume);
    }

    pub fn take_snapshot(&mut self) {
        self.send_command_chained(Command::TakeSnapshot);
    }

    pub fn move_cursor(&mut self, x: isize, y: isize) {
        self.send_command_chained(Command::MoveCursor(CursorPos { x, y }));
    }

    pub fn press(&mut self) {
        self.send_command_chained(Command::Pressed);
    }

    pub fn release(&mut self) {
        self.send_command_chained(Command::Released);
    }

    pub fn change_color(&mut self) {
        if let Some(color) = self.palette.next() {
            self.send_command_chained(Command::ChangeColor(color));
        }
    }

    pub fn needs_update(&self) -> bool {
        !self.commands.is_empty()
    }

    pub fn connect_command_sender(&mut self, chained: Box<dyn FnMut(Command)>) {
        self.chained_command_sender = Some(chained);
    }

    fn send_command_chained(&mut self, cmd: Command) {
        self.commands.push_back(cmd);

        if let Some(chained) = self.chained_command_sender.as_mut() {
            chained(cmd);
        }
    }
}

impl CommandSender for App {
    fn send_command(&mut self, cmd: Command) {
        self.commands.push_back(cmd);
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

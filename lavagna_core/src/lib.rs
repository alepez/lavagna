mod color;
pub mod doc;
mod painter;

use crate::color::*;
use crate::doc::{MutSketch, OwnedSketch};
use crate::painter::Painter;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq)]
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
    ChangeColor(CollabId, Color),
    MoveCursor(CollabId, CursorPos),
    Pressed(CollabId),
    Released(CollabId),
}

pub trait CommandSender {
    fn send_command(&mut self, cmd: Command);
}

pub struct Pen {
    cursor: Cursor,
    prev_cursor: Cursor,
    color: Color,
}

pub struct App {
    pen: Pen,
    commands: VecDeque<Command>,
    palette: ColorSelector,
    snapshots: Vec<OwnedSketch>,
    chained_command_sender: Option<Box<dyn FnMut(Command)>>,
    collab_id: CollabId,
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

        let pen = Pen {
            color,
            cursor: Cursor::default(),
            prev_cursor: Cursor::default(),
        };

        App {
            pen,
            commands: VecDeque::with_capacity(10),
            palette,
            snapshots: Vec::new(),
            chained_command_sender: Default::default(),
            collab_id: CollabId::default(),
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
                Command::ChangeColor(collab_id, color) => {
                    if collab_id == self.collab_id {
                        self.pen.color = color;
                    }
                }
                Command::MoveCursor(collab_id, pos) => {
                    if collab_id == self.collab_id {
                        self.pen.cursor.pos = pos;
                    }
                }
                Command::Pressed(collab_id) => {
                    if collab_id == self.collab_id {
                        self.pen.cursor.pressed = true;
                    }
                }
                Command::Released(collab_id) => {
                    if collab_id == self.collab_id {
                        self.pen.cursor.pressed = false;
                    }
                }
            }
        }

        let mut painter = Painter::new(sketch, self.pen.color);

        if self.pen.cursor.pressed && self.pen.prev_cursor.pressed {
            painter.draw_line(self.pen.prev_cursor.pos, self.pen.cursor.pos);
        } else {
            draw_current_color_icon(&mut painter);
        }

        self.pen.prev_cursor = self.pen.cursor;
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
        self.send_command_chained(Command::MoveCursor(self.collab_id, CursorPos { x, y }));
    }

    pub fn press(&mut self) {
        self.send_command_chained(Command::Pressed(self.collab_id));
    }

    pub fn release(&mut self) {
        self.send_command_chained(Command::Released(self.collab_id));
    }

    pub fn change_color(&mut self) {
        if let Some(color) = self.palette.next() {
            self.send_command_chained(Command::ChangeColor(self.collab_id, color));
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

    pub fn collab_id(&self) -> CollabId {
        self.collab_id
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

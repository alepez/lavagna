#![deny(clippy::all)]
#![forbid(unsafe_code)]

mod color;
pub mod doc;
mod painter;

use crate::color::*;
use crate::doc::{MutSketch, OwnedSketch};
use crate::painter::Painter;
use serde::{Deserialize, Serialize};
use std::borrow::BorrowMut;
use std::collections::{HashMap, VecDeque};

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct PenId(u32);

impl From<u32> for PenId {
    fn from(x: u32) -> Self {
        Self(x)
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum PenCommand {
    ChangeColor(Color),
    MoveCursor(CursorPos),
    Pressed,
    Released,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum Command {
    ClearAll,
    ResumeLastSnapshot,
    TakeSnapshot,
    PenCommand(PenId, PenCommand),
}

pub trait CommandSender {
    fn send_command(&mut self, cmd: Command);
}

#[derive(Default)]
struct Pen {
    cursor: Cursor,
    prev_cursor: Cursor,
    color: Color,
}

#[derive(Default)]
struct Pens(HashMap<PenId, Pen>);

impl Pens {
    fn select(&mut self, id: PenId) -> &mut Pen {
        self.0.entry(id).or_insert_with(Pen::default).borrow_mut()
    }
}

pub struct App {
    /// This instance pen id
    pen_id: PenId,
    /// All collaborative pens, included the owned one
    pens: Pens,
    /// A queue of commands to be performed
    commands: VecDeque<Command>,
    /// Palette of colors
    palette: ColorSelector,
    /// A history of snapshots
    snapshots: Vec<OwnedSketch>,
    /// A component in charge of sending commands to collaborators
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

impl App {
    pub fn new(pen_id: PenId) -> Self {
        App {
            pens: Pens::default(),
            commands: VecDeque::with_capacity(10),
            palette: ColorSelector::new(&PALETTE),
            snapshots: Vec::new(),
            chained_command_sender: Default::default(),
            pen_id,
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
                Command::ResumeLastSnapshot => {
                    if let Some(backup) = &self.snapshots.pop() {
                        sketch.copy_from(&backup.as_sketch());
                    }
                }
                Command::PenCommand(pen_id, cmd) => {
                    self.handle_pen_command(pen_id, cmd);
                }
            }
        }

        let mut painter = Painter::new(sketch);

        painter.set_color(self.pens.select(self.pen_id).color);
        draw_current_color_icon(&mut painter);

        for (_, pen) in self.pens.0.iter_mut() {
            painter.set_color(pen.color);

            if pen.cursor.pressed && pen.prev_cursor.pressed {
                painter.draw_line(pen.prev_cursor.pos, pen.cursor.pos);
            }

            pen.prev_cursor = pen.cursor;
        }
    }

    pub fn clear_all(&mut self) {
        self.send_command_chained(Command::ClearAll);
    }

    pub fn resume_last_snapshot(&mut self) {
        self.send_command_chained(Command::ResumeLastSnapshot);
    }

    pub fn take_snapshot(&mut self) {
        self.send_command_chained(Command::TakeSnapshot);
    }

    pub fn move_cursor(&mut self, x: isize, y: isize) {
        self.send_pen_command(PenCommand::MoveCursor(CursorPos { x, y }));
    }

    pub fn press(&mut self) {
        self.send_pen_command(PenCommand::Pressed);
    }

    pub fn release(&mut self) {
        self.send_pen_command(PenCommand::Released);
    }

    pub fn change_color(&mut self) {
        if let Some(color) = self.palette.next() {
            self.send_pen_command(PenCommand::ChangeColor(color));
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

    fn send_pen_command(&mut self, cmd: PenCommand) {
        let cmd = Command::PenCommand(self.pen_id, cmd);
        self.send_command_chained(cmd);
    }

    pub fn force_release(&mut self) {
        self.send_pen_command(PenCommand::Released);
    }

    fn handle_pen_command(&mut self, pen_id: PenId, cmd: PenCommand) {
        let pen = self.pens.select(pen_id);
        match cmd {
            PenCommand::ChangeColor(color) => {
                pen.color = color;
            }
            PenCommand::MoveCursor(pos) => {
                pen.cursor.pos = pos;
            }
            PenCommand::Pressed => {
                pen.cursor.pressed = true;
            }
            PenCommand::Released => {
                pen.cursor.pressed = false;
            }
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

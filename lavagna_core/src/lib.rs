#![deny(clippy::all)]
#![forbid(unsafe_code)]

use std::borrow::BorrowMut;
use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::color::*;
use crate::doc::{MutSketch, OwnedSketch};
use crate::painter::Painter;
use crate::ui::{Event, State};

mod color;
pub mod doc;
mod painter;
mod ui;

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
    ChangeSize(PenSize),
    MoveCursor(Cursor),
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
    size: PenSize,
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
    /// The (optional) user interface
    ui: Option<ui::Ui>,
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Cursor {
    pub pressed: bool,
    pub pos: CursorPos,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            pressed: false,
            pos: CursorPos { x: 0, y: 0 },
        }
    }
}

#[derive(Default, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CursorPos {
    pub x: isize,
    pub y: isize,
}

impl App {
    pub fn new(pen_id: PenId) -> Self {
        let palette = ColorSelector::new(&PALETTE);

        let ui = ui::Ui::new(State {
            full: true,
            color: palette.current_color(),
        });

        App {
            pens: Pens::default(),
            commands: VecDeque::with_capacity(10),
            palette,
            snapshots: Vec::new(),
            chained_command_sender: Default::default(),
            pen_id,
            ui: Some(ui),
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

        if let Some(ui) = &mut self.ui {
            let mut ui_state = ui.state;
            ui_state.color = self.pens.select(self.pen_id).color;

            ui.update(ui_state);
            ui.draw(&mut painter);
        }

        let local_pen = self.pens.select(self.pen_id);
        if let Some(ui) = &mut self.ui {
            if let Some(event) = ui.touch(&local_pen.cursor) {
                match event {
                    Event::ChangeColor => self.change_color(),
                    Event::ClearAll => self.clear_all(),
                    Event::ShrinkPen => self.shrink_pen(),
                    Event::GrowPen => self.grow_pen(),
                }
            }
        }

        for (_, pen) in self.pens.0.iter_mut() {
            painter.set_color(pen.color);
            painter.set_size(pen.size);

            if pen.prev_cursor.pressed {
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

    pub fn move_cursor(&mut self, cursor: Cursor) {
        self.send_pen_command(PenCommand::MoveCursor(cursor));
    }

    pub fn change_color(&mut self) {
        if let Some(color) = self.palette.next() {
            self.send_pen_command(PenCommand::ChangeColor(color));
        }
    }

    pub fn grow_pen(&mut self) {
        let mut size = self.my_pen().size;
        size.grow();
        self.send_pen_command(PenCommand::ChangeSize(size));
    }

    pub fn shrink_pen(&mut self) {
        let mut size = self.my_pen().size;
        size.shrink();
        self.send_pen_command(PenCommand::ChangeSize(size));
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

    fn handle_pen_command(&mut self, pen_id: PenId, cmd: PenCommand) {
        let pen = self.pens.select(pen_id);
        match cmd {
            PenCommand::ChangeColor(color) => {
                pen.color = color;
            }
            PenCommand::ChangeSize(size) => {
                pen.size = size;
            }
            PenCommand::MoveCursor(cursor) => {
                pen.cursor = cursor;
            }
        }
    }

    fn my_pen(&mut self) -> &mut Pen {
        self.pens.select(self.pen_id)
    }
}

impl CommandSender for App {
    fn send_command(&mut self, cmd: Command) {
        self.commands.push_back(cmd);
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PenSize(u32);

impl Default for PenSize {
    fn default() -> Self {
        Self(1)
    }
}

impl PenSize {
    fn grow(&mut self) {
        self.0 = (self.0 * 2).clamp(1, 32);
    }
    fn shrink(&mut self) {
        self.0 = (self.0 / 2).clamp(1, 32);
    }
}

use crate::color::{Color, WHITE};
use crate::painter::Painter;
use crate::{CursorPos, PenSize};

pub struct Ui {
    state: State,
}

pub struct State {
    pub color: Color,
}

impl Ui {
    pub fn new(state: State) -> Self {
        Self { state }
    }

    pub fn draw(&self, painter: &mut Painter) {
        draw_ui(painter, &self.state);
    }

    pub fn touch(&self) {}

    pub fn update(&mut self, state: State) {
        self.state = state;
    }
}

fn draw_ui(painter: &mut Painter, state: &State) {
    draw_icon_current_color(painter, &state.color);
    draw_icon_clear_all(painter);
    draw_icon_clear_change_color(painter, &state.color);
    draw_icon_clear_shrink_pen(painter);
    draw_icon_clear_grow_pen(painter);
}

fn draw_icon_clear_change_color(painter: &mut Painter, color: &Color) {
    let rect = Rect {
        x1: 0,
        y1: 0,
        x2: 100,
        y2: 100,
    };

    draw_rect(painter, &rect);

    painter.set_color(*color);

    for x in rect.x1..rect.x2 {
        for y in rect.y1..rect.y2 {
            painter.draw_pixel(CursorPos { x, y });
        }
    }
}

fn draw_icon_clear_all(painter: &mut Painter) {
    let rect = Rect {
        x1: 0,
        y1: 100,
        x2: 100,
        y2: 200,
    };
    draw_rect(painter, &rect);
}

fn draw_icon_clear_shrink_pen(painter: &mut Painter) {
    let rect = Rect {
        x1: 0,
        y1: 200,
        x2: 100,
        y2: 300,
    };
    draw_rect(painter, &rect);
}

fn draw_icon_clear_grow_pen(painter: &mut Painter) {
    let rect = Rect {
        x1: 0,
        y1: 300,
        x2: 100,
        y2: 400,
    };
    draw_rect(painter, &rect);
}

fn draw_icon_current_color(painter: &mut Painter, color: &Color) {
    const SQUARE_SIZE: isize = 10;

    painter.set_color(*color);

    for x in 0..SQUARE_SIZE {
        for y in 0..(SQUARE_SIZE - x) {
            painter.draw_pixel(CursorPos { x, y });
        }
    }
}

struct Rect {
    x1: isize,
    y1: isize,
    x2: isize,
    y2: isize,
}

fn draw_rect(painter: &mut Painter, rect: &Rect) {
    painter.set_color(WHITE);
    painter.set_size(PenSize(1));

    let a = CursorPos {
        x: rect.x1,
        y: rect.y1,
    };
    let b = CursorPos {
        x: rect.x2,
        y: rect.y1,
    };
    let c = CursorPos {
        x: rect.x2,
        y: rect.y2,
    };
    let d = CursorPos {
        x: rect.x1,
        y: rect.y2,
    };

    painter.draw_line(a, b);
    painter.draw_line(b, c);
    painter.draw_line(c, d);
    painter.draw_line(d, a);
}

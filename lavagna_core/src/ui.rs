use crate::color::{Color, WHITE};
use crate::painter::Painter;
use crate::{CursorPos, PenSize};

pub struct Ui {
    state: State,

    was_pressed: bool,

    change_color_btn: Button,
}

pub struct State {
    pub color: Color,
}

pub enum Event {
    ChangeColor,
}

struct Button {
    rect: Rect,
}

struct Rect {
    x1: isize,
    y1: isize,
    x2: isize,
    y2: isize,
}

impl Ui {
    pub fn new(state: State) -> Self {
        let change_color_btn = Button {
            rect: Rect {
                x1: 0,
                y1: 0,
                x2: 100,
                y2: 100,
            },
        };

        Self {
            state,
            was_pressed: false,
            change_color_btn,
        }
    }

    pub fn touch(&mut self, pos: &CursorPos, pressed: bool) -> Option<Event> {
        let clicked = self.was_pressed && !pressed;
        self.was_pressed = pressed;

        if !clicked {
            return None;
        }

        if is_cursor_inside_rect(pos, &self.change_color_btn.rect) {
            return Some(Event::ChangeColor);
        }

        None
    }

    pub fn update(&mut self, state: State) {
        self.state = state;
    }

    pub fn draw(&self, painter: &mut Painter) {
        let state = &self.state;

        self.draw_icon_current_color(painter, &state.color);
        self.draw_icon_clear_all(painter);
        self.draw_icon_clear_change_color(painter, &state.color);
        self.draw_icon_clear_shrink_pen(painter);
        self.draw_icon_clear_grow_pen(painter);
    }

    fn draw_icon_clear_change_color(&self, painter: &mut Painter, color: &Color) {
        let rect = &self.change_color_btn.rect;
        draw_rect(painter, rect);
        painter.set_color(*color);

        for x in rect.x1..rect.x2 {
            for y in rect.y1..rect.y2 {
                painter.draw_pixel(CursorPos { x, y });
            }
        }
    }

    fn draw_icon_clear_all(&self, painter: &mut Painter) {
        let rect = Rect {
            x1: 0,
            y1: 100,
            x2: 100,
            y2: 200,
        };
        draw_rect(painter, &rect);
    }

    fn draw_icon_clear_shrink_pen(&self, painter: &mut Painter) {
        let rect = Rect {
            x1: 0,
            y1: 200,
            x2: 100,
            y2: 300,
        };
        draw_rect(painter, &rect);
    }

    fn draw_icon_clear_grow_pen(&self, painter: &mut Painter) {
        let rect = Rect {
            x1: 0,
            y1: 300,
            x2: 100,
            y2: 400,
        };
        draw_rect(painter, &rect);
    }

    fn draw_icon_current_color(&self, painter: &mut Painter, color: &Color) {
        const SQUARE_SIZE: isize = 10;

        painter.set_color(*color);

        for x in 0..SQUARE_SIZE {
            for y in 0..(SQUARE_SIZE - x) {
                painter.draw_pixel(CursorPos { x, y });
            }
        }
    }
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

fn is_cursor_inside_rect(cursor: &CursorPos, rect: &Rect) -> bool {
    (cursor.x > rect.x1) && (cursor.x < rect.x2) && (cursor.y > rect.y1) && (cursor.y < rect.y2)
}

use crate::color::{Color, WHITE};
use crate::painter::Painter;
use crate::ui::Event::Handled;
use crate::{Cursor, CursorPos, PenSize};

pub struct Ui {
    pub state: State,

    was_pressed: bool,

    change_color_btn: Button,
    clear_all_btn: Button,
    shrink_pen_btn: Button,
    grow_pen_btn: Button,
}

#[derive(Copy, Clone)]
pub struct State {
    pub full: bool,
    pub color: Color,
}

pub enum Event {
    ChangeColor,
    ClearAll,
    ShrinkPen,
    GrowPen,
    Handled,
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

impl Button {
    fn new(x: isize, y: isize, width: isize, height: isize) -> Self {
        let rect = Rect {
            x1: x,
            y1: y,
            x2: x + width,
            y2: y + height,
        };
        Self { rect }
    }

    fn pressed(&self, cursor: &CursorPos) -> bool {
        is_cursor_inside_rect(cursor, &self.rect)
    }
}

impl Ui {
    pub fn new(state: State) -> Self {
        let change_color_btn = Button::new(0, 0, 100, 100);
        let clear_all_btn = Button::new(0, 100, 100, 100);
        let shrink_pen_btn = Button::new(0, 200, 100, 100);
        let grow_pen_btn = Button::new(0, 300, 100, 100);

        Self {
            state,
            was_pressed: false,
            change_color_btn,
            clear_all_btn,
            shrink_pen_btn,
            grow_pen_btn,
        }
    }

    pub fn touch(&mut self, cursor: &Cursor) -> Option<Event> {
        let pressed = cursor.pressed;
        let clicked = self.was_pressed && !pressed;
        self.was_pressed = pressed;

        if self.change_color_btn.pressed(&cursor.pos) {
            return if clicked {
                Some(Event::ChangeColor)
            } else {
                Some(Handled)
            };
        }

        if self.clear_all_btn.pressed(&cursor.pos) {
            return if clicked {
                Some(Event::ClearAll)
            } else {
                Some(Handled)
            };
        }

        if self.shrink_pen_btn.pressed(&cursor.pos) {
            return if clicked {
                Some(Event::ShrinkPen)
            } else {
                Some(Handled)
            };
        }

        if self.grow_pen_btn.pressed(&cursor.pos) {
            return if clicked {
                Some(Event::GrowPen)
            } else {
                Some(Handled)
            };
        }

        None
    }

    pub fn update(&mut self, state: State) {
        self.state = state;
    }

    pub fn draw(&self, painter: &mut Painter) {
        let state = &self.state;

        if state.full {
            self.draw_clear_all_button(painter);
            self.draw_change_color_button(painter, &state.color);
            self.draw_shrink_pen_button(painter);
            self.draw_grow_pen_button(painter);
        } else {
            self.draw_icon_current_color(painter, &state.color);
        }
    }

    fn draw_change_color_button(&self, painter: &mut Painter, color: &Color) {
        let rect = &self.change_color_btn.rect;
        draw_rect(painter, rect);
        painter.set_color(*color);

        for x in rect.x1..rect.x2 {
            for y in rect.y1..rect.y2 {
                painter.draw_pixel(CursorPos { x, y });
            }
        }
    }

    fn draw_clear_all_button(&self, painter: &mut Painter) {
        let rect = &self.clear_all_btn.rect;
        painter.set_color(WHITE);
        draw_rect(painter, rect);
    }

    fn draw_shrink_pen_button(&self, painter: &mut Painter) {
        let rect = &self.shrink_pen_btn.rect;
        painter.set_color(WHITE);
        draw_rect(painter, rect);
    }

    fn draw_grow_pen_button(&self, painter: &mut Painter) {
        let rect = &self.grow_pen_btn.rect;
        painter.set_color(WHITE);
        draw_rect(painter, rect);
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

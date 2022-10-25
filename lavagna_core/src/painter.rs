#![deny(clippy::all)]
#![forbid(unsafe_code)]

use crate::color::*;
use crate::{CursorPos, MutSketch, PenSize};
use line_drawing::{Bresenham, BresenhamCircle};
use std::ops::Range;

pub struct Painter<'a> {
    sketch: MutSketch<'a>,
    color: Color,
    size: u32,
}

impl<'a> Painter<'a> {
    pub fn new(sketch: MutSketch<'a>) -> Self {
        let color = WHITE;
        let size = 1;
        Painter {
            sketch,
            color,
            size,
        }
    }

    pub fn draw_line(&mut self, from: CursorPos, to: CursorPos) {
        for (x_offset, y_offset) in BresenhamCircle::new(0, 0, self.size as isize) {
            let from = CursorPos {
                x: from.x + x_offset,
                y: from.y + y_offset,
            };
            let to = CursorPos {
                x: to.x + x_offset,
                y: to.y + y_offset,
            };
            self.draw_line_1px(from, to);
        }
    }

    fn draw_line_1px(&mut self, from: CursorPos, to: CursorPos) {
        Bresenham::new((from.x, from.y), (to.x, to.y)).for_each(|(x, y)| {
            self.draw_pixel(CursorPos { x, y });
        });
    }

    pub fn draw_pixel(&mut self, pos: CursorPos) {
        let CursorPos { x, y } = pos;

        let w = self.sketch.size.width as isize;

        let pix_index = w * y + x;
        let pix_max = self.sketch.frame.len() as isize;

        if pix_index < 0 || pix_index > pix_max {
            return;
        }

        let pix_index = pix_index as usize;

        if let Some(pix) = self.sketch.frame.chunks_exact_mut(4).nth(pix_index) {
            pix.copy_from_slice(self.color.as_bytes());
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn set_size(&mut self, size: PenSize) {
        self.size = size.0;
    }
}

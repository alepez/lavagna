#![deny(clippy::all)]
#![forbid(unsafe_code)]

use crate::color::*;
use crate::{CursorPos, MutSketch, PenSize};
use line_drawing::Bresenham;
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
        for x_offset in calculate_size_range(self.size) {
            for y_offset in calculate_size_range(self.size) {
                Bresenham::new(
                    (from.x + x_offset, from.y + y_offset),
                    (to.x + x_offset, to.y + y_offset),
                )
                .for_each(|(x, y)| {
                    self.draw_pixel(CursorPos { x, y });
                });
            }
        }
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

fn calculate_size_range(size: u32) -> Range<isize> {
    if size == 1 {
        return 0..1;
    }
    let size = size as isize;
    let half_size = size / 2;
    (-half_size)..(half_size)
}

#[cfg(test)]
mod tests {
    use crate::painter::calculate_size_range;

    #[test]
    fn test_size_range() {
        assert_eq!((0..1), calculate_size_range(1));
        assert_eq!((-1..1), calculate_size_range(2));
        assert_eq!((-1..1), calculate_size_range(3));
        assert_eq!((-2..2), calculate_size_range(4));
        assert_eq!((-2..2), calculate_size_range(5));
    }
}

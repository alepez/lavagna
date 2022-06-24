use crate::app::color::*;
use crate::app::CursorPos;
use crate::MutSketch;
use line_drawing::Bresenham;

pub struct Painter<'a> {
    sketch: MutSketch<'a>,
    color: Color,
}

impl<'a> Painter<'a> {
    pub fn new(sketch: MutSketch<'a>, color: Color) -> Self {
        Painter { sketch, color }
    }

    pub fn draw_line(&mut self, from: CursorPos, to: CursorPos) {
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
}

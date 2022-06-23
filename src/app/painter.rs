use line_drawing::Bresenham;
use crate::app::CursorPos;
use crate::app::color::*;
use crate::Sketch;

pub struct Painter<'a> {
    sketch: Sketch<'a>,
    color: Color,
}

impl<'a> Painter<'a> {
    pub fn new(sketch: Sketch<'a>, color: Color) -> Self {
        Painter { sketch, color }
    }

    pub fn draw_line(&mut self, from: CursorPos, to: CursorPos) {
        Bresenham::new((from.x, from.y), (to.x, to.y)).for_each(|(x, y)| {
            self.draw_pixel(CursorPos { x, y });
        });
    }

    pub fn draw_pixel(&mut self, pos: CursorPos) {
        let CursorPos { x, y } = pos;

        let pix_index = (self.sketch.size.width * y + x) as usize;

        if let Some(pix) = self.sketch.frame
            .chunks_exact_mut(4)
            .nth(pix_index) {
            pix.copy_from_slice(self.color.as_bytes());
        }
    }
}
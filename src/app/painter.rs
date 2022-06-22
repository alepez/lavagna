use crate::app::{Canvas, CursorPos};
use crate::app::line::BresenhamLine;

pub struct Painter<'a> {
    frame: &'a mut [u8],
    width: isize,
}

impl<'a> Painter<'a> {
    pub fn new(frame: &'a mut [u8], canvas: &Canvas) -> Self {
        Painter { frame, width: canvas.width }
    }

    pub fn draw_line(&mut self, from: CursorPos, to: CursorPos) {
        BresenhamLine::new(from, to).for_each(|p| {
            self.draw_pixel(p);
        });
    }

    pub fn draw_pixel(&mut self, pos: CursorPos) {
        let CursorPos { x, y } = pos;

        let pix_index = (self.width * y + x) as usize;

        if let Some(pix) = self.frame
            .chunks_exact_mut(4)
            .nth(pix_index) {
            let color = [0xff, 0xff, 0xff, 0xff];
            pix.copy_from_slice(&color);
        }
    }

    pub fn clear(&mut self) {
        self.frame.fill(0x00);
    }
}
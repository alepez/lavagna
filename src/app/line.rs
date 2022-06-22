use crate::app::CursorPos;

pub struct BresenhamLine {
    error: isize,
    x: isize,
    y: isize,
    sx: isize,
    sy: isize,
    dx: isize,
    dy: isize,
    to: CursorPos,
}

impl BresenhamLine {
    pub fn new(from: CursorPos, to: CursorPos) -> BresenhamLine {
        let dx = (to.x - from.x).abs();
        let dy = (to.y - from.y).abs();

        BresenhamLine {
            error: (if dx > dy { dx } else { -dy }) / 2,
            x: from.x,
            y: from.y,
            sx: if from.x < to.x { 1 } else { -1 },
            sy: if from.y < to.y { 1 } else { -1 },
            dx,
            dy,
            to,
        }
    }
}

impl Iterator for BresenhamLine {
    type Item = CursorPos;

    fn next(&mut self) -> Option<Self::Item> {
        if (self.x != self.to.x) || (self.y != self.to.y) {
            let next = CursorPos { x: self.x, y: self.y };

            let e = self.error;

            if e > -self.dx {
                self.error -= self.dy;
                self.x += self.sx;
            }

            if e < self.dy {
                self.error += self.dx;
                self.y += self.sy;
            }

            Some(next)
        } else {
            None
        }
    }
}
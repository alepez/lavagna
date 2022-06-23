#[derive(Copy, Clone)]
pub struct Color([u8; 4]);

pub const WHITE: Color = Color([0xff, 0xff, 0xff, 0xff]);
pub const BLUE: Color = Color([0x00, 0x6f, 0xff, 0xff]);
pub const LIGHT_BLUE: Color = Color([0x13, 0xf4, 0xef, 0xff]);
pub const GREEN: Color = Color([0x68, 0xff, 0x00, 0xff]);
pub const YELLOW: Color = Color([0xfa, 0xff, 0x00, 0xff]);
pub const ORANGE: Color = Color([0xff, 0xbf, 0x00, 0xff]);
pub const RED: Color = Color([0xff, 0x00, 0x5c, 0xff]);

impl<'a> Color {
    pub fn as_bytes(&'a self) -> &'a [u8] {
        &self.0
    }
}

pub const PALETTE: [Color; 7] = [WHITE, BLUE, LIGHT_BLUE, GREEN, YELLOW, ORANGE, RED];

pub struct ColorSelector {
    index: usize,
    palette: &'static [Color],
}

impl ColorSelector {
    pub fn new(palette: &'static [Color]) -> Self {
        Self { index: 0, palette }
    }
}

impl Iterator for ColorSelector {
    type Item = Color;

    fn next(&mut self) -> Option<Self::Item> {
        let color = self.palette[self.index];
        self.index = (self.index + 1) % self.palette.len();
        Some(color)
    }
}

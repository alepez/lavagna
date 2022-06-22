pub struct Color([u8; 4]);

pub const WHITE: Color = Color([0xff, 0xff, 0xff, 0xff]);

impl<'a> Color {
    pub fn as_bytes(&'a self) -> &'a [u8] {
        &self.0
    }
}
pub struct Sketch<'a> {
    pub size: Size,
    pub frame: &'a mut [u8],
}

impl<'a> Sketch<'a> {
    pub fn new(frame: &'a mut [u8], width: impl TryInto<isize>, height: impl TryInto<isize>) -> Self {
        let width = width.try_into().ok().unwrap();
        let height = height.try_into().ok().unwrap();

        Self {
            size: Size { width, height },
            frame,
        }
    }
}

pub struct Size {
    pub(crate) width: isize,
    #[allow(dead_code)]
    pub(crate) height: isize,
}


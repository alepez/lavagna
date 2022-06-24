pub struct Sketch<'a> {
    pub size: Size,
    pub frame: &'a [u8],
}

pub struct MutSketch<'a> {
    pub size: Size,
    pub frame: &'a mut [u8],
}

impl<'a> MutSketch<'a> {
    pub fn new(
        frame: &'a mut [u8],
        width: impl TryInto<usize>,
        height: impl TryInto<usize>,
    ) -> Self {
        let width = width.try_into().ok().unwrap();
        let height = height.try_into().ok().unwrap();

        Self {
            size: Size { width, height },
            frame,
        }
    }

    pub fn to_owned(&self) -> OwnedSketch {
        OwnedSketch {
            size: self.size,
            frame: self.frame.to_owned(),
        }
    }

    pub fn copy_from(&mut self, other: &Sketch<'_>) {
        let min_h = usize::min(self.size.height, other.size.height);
        let min_w = usize::min(self.size.width, other.size.width);
        let dst_w = self.size.width;
        let src_w = other.size.width;
        let dst = &mut self.frame;
        let src = &other.frame;

        for y in 0..min_h {
            let dst_begin = dst_w * y;
            let dst_end = dst_begin + min_w;
            let src_begin = src_w * y;
            let src_end = src_begin + min_w;
            let dst_range = (4 * dst_begin)..(4 * dst_end);
            let src_range = (4 * src_begin)..(4 * src_end);
            dst[dst_range].copy_from_slice(&src[src_range]);
        }
    }
}

pub struct OwnedSketch {
    pub size: Size,
    pub frame: Vec<u8>,
}

impl OwnedSketch {
    pub fn as_sketch(&self) -> Sketch {
        Sketch {
            size: self.size,
            frame: self.frame.as_slice(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Size {
    pub(crate) width: usize,
    #[allow(dead_code)]
    pub(crate) height: usize,
}

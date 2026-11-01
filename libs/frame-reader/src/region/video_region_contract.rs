use memmap2::Mmap;

/// How the pixels of a picture are laid out, a row after another with no padding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    /// Red, green and blue, a byte each.
    Rgb24,
    /// Brightness only, a byte.
    Gray8,
}

impl PixelFormat {
    pub fn bytes_per_pixel(self) -> usize {
        match self {
            PixelFormat::Rgb24 => 3,
            PixelFormat::Gray8 => 1,
        }
    }
}

/// What the list says about one picture. `index` is its place in the file and means nothing
/// outside it: with a variable frame rate, time is the only address that holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    pub index: u64,
    /// When the picture is shown, in milliseconds from the start of the source.
    pub pts_ms: u64,
    pub duration_ms: u64,
    pub is_key_frame: bool,
}

/// The pictures of a video (or of a stretch of it), mapped from the file the host decoded.
#[derive(Debug)]
pub struct VideoRegion {
    pub width: usize,
    pub height: usize,
    pub format: PixelFormat,
    /// Where the first picture is in the source, in milliseconds.
    pub start_ms: u64,
    pub end_ms: u64,
    pub frames: Vec<Frame>,
    pub(crate) map: Option<Mmap>,
    pub(crate) offset: usize,
}

impl VideoRegion {
    /// The bytes of one picture: `width * height * bytes per pixel` of them, a row after another.
    /// `None` when there is no such picture.
    pub fn pixels(&self, index: usize) -> Option<&[u8]> {
        let size = self.width * self.height * self.format.bytes_per_pixel();
        let from = self.offset.checked_add(index.checked_mul(size)?)?;
        self.map.as_ref()?.get(from..from.checked_add(size)?)
    }
}

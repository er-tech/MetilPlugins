/// One thing the detector found in a picture: its class (the number in the COCO set), how sure it
/// is, and the box around it in pixels of the picture, corner to corner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Detection {
    pub class: usize,
    pub score: f32,
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Detection {
    pub fn area(&self) -> f32 {
        (self.x2 - self.x1).max(0.0) * (self.y2 - self.y1).max(0.0)
    }
}

/// Where a line of text is in a picture: the corners in pixels of the picture, and how sure the
/// detector is (0 to 1).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextBox {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub score: f32,
}

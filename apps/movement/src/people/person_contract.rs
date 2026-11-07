/// How many keypoints a body has: the 17 of the COCO set, in its order.
pub const KEYPOINTS: usize = 17;

/// A keypoint counts when the model is at least this sure of it.
pub const MIN_SCORE: f32 = 0.3;

/// The places of the keypoints in the list of a person.
pub mod joint {
    pub const LEFT_SHOULDER: usize = 5;
    pub const RIGHT_SHOULDER: usize = 6;
    pub const LEFT_WRIST: usize = 9;
    pub const RIGHT_WRIST: usize = 10;
    pub const LEFT_HIP: usize = 11;
    pub const RIGHT_HIP: usize = 12;
    pub const LEFT_ANKLE: usize = 15;
    pub const RIGHT_ANKLE: usize = 16;
}

/// One keypoint: where it is in pixels of the picture, and how sure the model is of it (0 to 1).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Keypoint {
    pub x: f32,
    pub y: f32,
    pub score: f32,
}

/// One person: the keypoints of the body, the box round it (`x1, y1, x2, y2` in pixels) and how sure
/// the model is that it is a person.
#[derive(Debug, Clone, PartialEq)]
pub struct Person {
    pub keypoints: [Keypoint; KEYPOINTS],
    pub bbox: [f32; 4],
    pub score: f32,
}

impl Person {
    /// The keypoint number `joint`, when the model is sure enough of it.
    pub fn joint(&self, joint: usize) -> Option<Keypoint> {
        let keypoint = self.keypoints[joint];
        (keypoint.score >= MIN_SCORE).then_some(keypoint)
    }

    /// The middle of two joints (the shoulders, the hips), when both are there.
    pub fn middle(&self, first: usize, second: usize) -> Option<(f32, f32)> {
        let (a, b) = (self.joint(first)?, self.joint(second)?);
        Some(((a.x + b.x) / 2.0, (a.y + b.y) / 2.0))
    }

    /// The height of the box, in pixels.
    pub fn height(&self) -> f32 {
        (self.bbox[3] - self.bbox[1]).max(1.0)
    }

    /// The width of the box, in pixels.
    pub fn width(&self) -> f32 {
        (self.bbox[2] - self.bbox[0]).max(1.0)
    }

    /// The middle of the box.
    pub fn center(&self) -> (f32, f32) {
        (
            (self.bbox[0] + self.bbox[2]) / 2.0,
            (self.bbox[1] + self.bbox[3]) / 2.0,
        )
    }
}


/// Bounds for describing how far the fragment shader of a shape will reach, should be bigger than the shape unless you want to clip it
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Frame {
    pub half_width: f32,
    pub half_height: f32, // todo: it probably makes sense for this to be the full width instead...
}

impl Frame {
    const DEFAULT: Self = Self::square(1.0);

    pub const fn square(radius: f32) -> Self {
        Self {
            half_height: radius,
            half_width: radius,
        }
    }
}

impl Default for Frame {
    fn default() -> Self {
        Self::DEFAULT
    }
}
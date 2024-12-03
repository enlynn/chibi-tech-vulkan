use crate::math::{Float, float_is_zero, rand_float, rand_float_in_range};

#[derive(Debug, Copy, Clone)]
pub struct Float4 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
    pub w: Float,
}

impl Float4 {
    pub fn new(x: Float, y: Float, z: Float, w: Float) -> Float4 {
        Float4 { x, y, z, w }
    }

    pub fn fill(v: Float) -> Float4 {
        return Self::new(v, v, v, v);
    }

    pub fn zero() -> Float4 {
        return Self::fill(0.0);
    }

    pub fn one() -> Float4 {
        return Self::fill(1.0);
    }
}

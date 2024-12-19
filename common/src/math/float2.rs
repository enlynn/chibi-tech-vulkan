use crate::math::Float;

#[derive(Copy, Clone, Debug)]
pub struct Float2 {
    pub x: Float,
    pub y: Float,
}

impl Float2 {
    pub fn new(x: Float, y: Float) -> Float2 {
        Float2 { x, y }
    }

    pub fn fill(v: Float) -> Float2 {
        return Self::new(v, v);
    }

    pub fn zero() -> Float2 {
        return Self::fill(0.0);
    }

    pub fn one() -> Float2 {
        return Self::fill(1.0);
    }
}

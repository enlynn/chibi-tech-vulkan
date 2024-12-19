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

    pub fn dot(&self, v: Float4) -> Float {
        self.x * v.x + self.y * v.y + self.z * v.z + self.w * v.w
    }

    pub fn pack_unorm_u32(&self) -> u32 {
        let x = (self.x.clamp(0.0, 1.0) * 255.0) as u8;
        let y = (self.y.clamp(0.0, 1.0) * 255.0) as u8;
        let z = (self.z.clamp(0.0, 1.0) * 255.0) as u8;
        let w = (self.w.clamp(0.0, 1.0) * 255.0) as u8;

        #[repr(C)]
        union PackedByte {
            array: [u8; 4],
            val:   u32,
        }

        let packed = PackedByte{ array: [x, y, z, w] };
        return unsafe { packed.val };
    }
}

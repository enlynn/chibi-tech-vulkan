use std::ops;

use crate::math::{Float, float_is_zero, rand_float, rand_float_in_range};

#[derive(Copy, Clone, Debug)]
pub struct Float3 {
    pub x: Float,
    pub y: Float,
    pub z: Float,
}

impl Float3 {
    pub fn new(x: Float, y: Float, z: Float) -> Float3 {
        Float3 { x, y, z }
    }

    pub fn fill(v: Float) -> Float3 {
        return Self::new(v, v, v);
    }

    pub fn zero() -> Float3 {
        return Self::fill(0.0);
    }

    pub fn one() -> Float3 {
        return Self::fill(1.0);
    }

    pub fn squared_length(self) -> Float {
        (self.x * self.x) + (self.y * self.y) + (self.z * self.z)
    }

    pub fn is_zero(self) -> bool {
        return float_is_zero(self.x) && float_is_zero(self.y) && float_is_zero(self.z);
    }

    pub fn length(self) -> Float {
        let sq_len = self.squared_length();
        if float_is_zero(sq_len) {
            0.0
        } else {
            sq_len.sqrt()
        }
    }

    pub fn unit(self) -> Float3 {
        let len = self.length();
        if !float_is_zero(len) {
            self / self.length()
        } else {
            Float3::zero()
        }
    }

    pub fn dot(&self, v: Float3) -> Float {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn cross(self, v: Float3) -> Float3 {
        Float3::new(
            self.y * v.z - self.z * v.y,
            self.z * v.x - self.x * v.z,
            self.x * v.y - self.y * v.x,
        )
    }

    // Random based functions
    pub fn rand() -> Float3 {
        return Float3 {
            x: rand_float(),
            y: rand_float(),
            z: rand_float(),
        };
    }

    pub fn rand_in_range(min: Float, max: Float) -> Float3 {
        return Float3 {
            x: rand_float_in_range(min, max),
            y: rand_float_in_range(min, max),
            z: rand_float_in_range(min, max),
        };
    }

    pub fn rand_in_unit_sphere() -> Float3 {
        // Use "rejection" as a way of determining if the random point is within the sphere,
        // If the Length is outside the unit sphere, then it is an invalid point.

        // TODO: raytracing in a weekend mentions using analytical methods instead of using rejection

        const MAX_ITERATIONS: usize = 100;
        for _ in 0..MAX_ITERATIONS {
            let p = Float3::rand_in_range(-1.0, 1.0);
            if p.squared_length() < 1.0 {
                return p;
            }
        }

        return Float3::zero();
    }

    // Finds a normalized vector on a unit sphere
    pub fn rand_unit_vector() -> Float3 {
        Float3::rand_in_unit_sphere().unit()
    }

    // note: fill out swizzles as needed
    pub fn xxx(&self) -> Float3 { Float3::fill(self.x) }
    pub fn yyy(&self) -> Float3 { Float3::fill(self.y) }
    pub fn zzz(&self) -> Float3 { Float3::fill(self.z) }
    pub fn xxy(&self) -> Float3 { Float3::new(self.x, self.x, self.y) }
    pub fn xxz(&self) -> Float3 { Float3::new(self.x, self.x, self.z) }
    pub fn yxx(&self) -> Float3 { Float3::new(self.y, self.x, self.x) }
    pub fn zxx(&self) -> Float3 { Float3::new(self.y, self.x, self.x) }
}

impl PartialEq<Float3> for Float3 {
    fn eq(&self, other: &Float3) -> bool {
        // TODO: probably not a good idea to compare floats directly.
        return self.x == other.x && self.y == other.y && self.z == other.z;
    }
}

impl ops::Add<Float3> for Float3 {
    type Output = Float3;
    fn add(self, v: Float3) -> Float3 {
        Float3::new(self.x + v.x, self.y + v.y, self.z + v.z)
    }
}

impl ops::Sub<Float3> for Float3 {
    type Output = Float3;
    fn sub(self, v: Float3) -> Float3 {
        Float3::new(self.x - v.x, self.y - v.y, self.z - v.z)
    }
}

impl ops::Mul<Float3> for Float3 {
    type Output = Float3;
    fn mul(self, v: Float3) -> Float3 {
        Float3::new(self.x * v.x, self.y * v.y, self.z * v.z)
    }
}

impl ops::Div<Float3> for Float3 {
    type Output = Float3;
    fn div(self, v: Float3) -> Float3 {
        Float3::new(self.x / v.x, self.y / v.y, self.z / v.z)
    }
}

impl ops::Mul<Float> for Float3 {
    type Output = Float3;
    fn mul(self, f: Float) -> Float3 {
        Float3::new(self.x * f, self.y * f, self.z * f)
    }
}

impl ops::Mul<Float3> for Float {
    type Output = Float3;
    fn mul(self, f: Float3) -> Float3 {
        Float3::new(self * f.x, self * f.y, self * f.z)
    }
}

impl ops::Div<Float> for Float3 {
    type Output = Float3;
    fn div(self, f: Float) -> Float3 {
        Float3::new(self.x / f, self.y / f, self.z / f)
    }
}

impl ops::Neg for Float3 {
    type Output = Float3;
    fn neg(self) -> Float3 {
        Float3::new(-self.x, -self.y, -self.z)
    }
}

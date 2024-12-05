use std::convert::identity;

use crate::math::{
    {Float, float_is_zero, rand_float, rand_float_in_range},
    float4::*,
    float3::*,
};

use super::degrees_to_radians;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct _Cols {
    c0: Float4,
    c1: Float4,
    c2: Float4,
    c3: Float4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union Float4x4 {
    _cols: _Cols,
    _data: [[f32; 4]; 4],
}

// Default returns an identity matrix
impl Default for Float4x4 {
    fn default() -> Self {
        Self{
            _data: [
                [ 1.0, 0.0, 0.0, 0.0 ],
                [ 0.0, 1.0, 0.0, 0.0 ],
                [ 0.0, 0.0, 1.0, 0.0 ],
                [ 0.0, 0.0, 0.0, 1.0 ],
            ]
        }
    }
}

impl Float4x4 {
    pub fn new() -> Self {
        Self::identity()
    }

    pub fn diag(val: f32) -> Self {
        Self{
            _data: [
                [ val, 0.0, 0.0, 0.0 ],
                [ 0.0, val, 0.0, 0.0 ],
                [ 0.0, 0.0, val, 0.0 ],
                [ 0.0, 0.0, 0.0, val ],
            ]
        }
    }

    pub fn identity() -> Self {
        Self::default()
    }

    pub fn get_uniform_scale_matrix(scale: f32) -> Self {
        Self::get_scale_matrix(scale, scale, scale)
    }

    pub fn get_scale_matrix(scale_x: f32, scale_y: f32, scale_z: f32) -> Self {
        Self{
            _data: [
                [ scale_x, 0.0,     0.0, 0.0 ],
                [ 0.0, scale_y,     0.0, 0.0 ],
                [ 0.0,     0.0, scale_z, 0.0 ],
                [ 0.0,     0.0,     0.0, 1.0 ],
            ]
        }
    }

    pub fn get_translate_matrix(tranlsate_vector: Float4) -> Self {
        let mut result = Float4x4::default();

        unsafe {
            result._data[3][0] = tranlsate_vector.x;
            result._data[3][1] = tranlsate_vector.y;
            result._data[3][2] = tranlsate_vector.z;
        }

        return result;
    }

    // Get a Rotation Matrix using an angle about the X-Axis
    pub fn get_rotate_x_matrix(theta_degrees: f32) -> Self {
        let mut result = Float4x4::default();

        let radians = degrees_to_radians(theta_degrees);
        let c = radians.cos();
        let s = radians.sin();

        unsafe {
            result._data[0][0] = 1.0;
            result._data[0][1] = 0.0;
            result._data[0][2] = 0.0;
            result._data[0][3] = 0.0;

            result._data[1][0] = 0.0;
            result._data[1][1] = c;
            result._data[1][2] = s;
            result._data[1][3] = 0.0;

            result._data[2][0] = 0.0;
            result._data[2][1] = -s;
            result._data[2][2] = c;
            result._data[2][3] = 0.0;

            result._data[3][0] = 0.0;
            result._data[3][1] = 0.0;
            result._data[3][2] = 0.0;
            result._data[3][3] = 1.0;
        }

        return result;
    }

    // Get a Rotation Matrix using an angle about the X-Axis
    pub fn get_rotate_y_matrix(theta_degrees: f32) -> Self {
        let mut result = Float4x4::default();

        let radians = degrees_to_radians(theta_degrees);
        let c = radians.cos();
        let s = radians.sin();

        unsafe {
            result._data[0][0] = c;
            result._data[0][1] = 0.0;
            result._data[0][2] = -s;
            result._data[0][3] = 0.0;

            result._data[1][0] = 0.0;
            result._data[1][1] = 1.0;
            result._data[1][2] = 0.0;
            result._data[1][3] = 0.0;

            result._data[2][0] = s;
            result._data[2][1] = 0.0;
            result._data[2][2] = c;
            result._data[2][3] = 0.0;

            result._data[3][0] = 0.0;
            result._data[3][1] = 0.0;
            result._data[3][2] = 0.0;
            result._data[3][3] = 1.0;
        }

        return result;
    }

    // Get a Rotation Matrix using an angle about the X-Axis
    pub fn get_rotate_z_matrix(theta_degrees: f32) -> Self {
        let mut result = Float4x4::default();

        let radians = degrees_to_radians(theta_degrees);
        let c = radians.cos();
        let s = radians.sin();

        unsafe {
            result._data[0][0] = c;
            result._data[0][1] = s;
            result._data[0][2] = 0.0;
            result._data[0][3] = 0.0;

            result._data[1][0] = -s;
            result._data[1][1] = c;
            result._data[1][2] = 0.0;
            result._data[1][3] = 0.0;

            result._data[2][0] = 0.0;
            result._data[2][1] = 0.0;
            result._data[2][2] = 1.0;
            result._data[2][3] = 0.0;

            result._data[3][0] = 0.0;
            result._data[3][1] = 0.0;
            result._data[3][2] = 0.0;
            result._data[3][3] = 1.0;
        }

        return result;
    }

    // Get a Rotation Matrix using an angle about an arbitrary Axis
    pub fn get_rotation_matrix(theta_degrees: f32, mut rotation_axis: Float3) -> Self {
        let mut result = Float4x4::default();

        let radians = degrees_to_radians(theta_degrees);
        let c = radians.cos();
        let s = radians.sin();
        let d = 1.0 - c;

        rotation_axis = rotation_axis.unit();

        let x    = d * rotation_axis.x;
        let y    = d * rotation_axis.y;
        let z    = d * rotation_axis.z;
        let axay = x * rotation_axis.y;
        let axaz = x * rotation_axis.z;
        let ayaz = y * rotation_axis.z;

        unsafe {
            result._data[0][0] = c    + x * rotation_axis.x;
            result._data[0][1] = axay + s * rotation_axis.z;
            result._data[0][2] = axaz - s * rotation_axis.y;
            result._data[0][3] = 0.0;

            result._data[1][0] = axay - s * rotation_axis.z;
            result._data[1][1] = c    + y * rotation_axis.y;
            result._data[1][2] = ayaz + s * rotation_axis.x;
            result._data[1][3] = 0.0;

            result._data[2][0] = axaz + s * rotation_axis.y;
            result._data[2][1] = ayaz - s * rotation_axis.x;
            result._data[2][2] = c    + z * rotation_axis.z;
            result._data[2][3] = 0.0;

            result._data[3][0] = 0.0;
            result._data[3][1] = 0.0;
            result._data[3][2] = 0.0;
            result._data[3][3] = 1.0;
        }

        return result;
    }

    // get a Right-Handed Look-At Matrix
    pub fn get_look_at_matrix(eye_position: Float3, eye_look_at_point: Float3, mut up_vector: Float3) -> Self {
        let mut result = Float4x4::default();

        let mut f = eye_look_at_point - eye_position;
        f = f.unit();

        up_vector = up_vector.unit();

        let mut s = f.cross(up_vector);
        s = s.unit();

        let u = s.cross(f);

        unsafe {
            result._data[0][0] =  s.x;
            result._data[0][1] =  u.x;
            result._data[0][2] = -f.x;
            result._data[0][3] =  0.0;

            result._data[1][0] =  s.y;
            result._data[1][1] =  u.y;
            result._data[1][2] = -f.y;
            result._data[1][3] =  0.0;

            result._data[2][0] =  s.z;
            result._data[2][1] =  u.z;
            result._data[2][2] = -f.z;
            result._data[2][3] =  0.0;

            result._data[3][0] = -s.dot(eye_position);
            result._data[3][1] = -u.dot(eye_position);
            result._data[3][2] =  f.dot(eye_position);
            result._data[3][3] =  1.0;
        }

        return result;
    }

    // get a Right-Handed Perspective Matrix
    pub fn get_perspective_matrix(field_of_view: f32, aspect_ratio: f32, near_plane: f32, far_plane: f32) -> Self {
        let mut result = Float4x4::default();

        let radians   = degrees_to_radians(field_of_view);
        let cotangent = 1.0 / (radians * 0.5).tan();

        unsafe {
            result._data[0][0] = cotangent / aspect_ratio;
            result._data[1][1] = cotangent;
            result._data[2][3] = -1.0;
            result._data[2][2] = (near_plane + far_plane) / (near_plane - far_plane);
            result._data[3][2] = (2.0 * near_plane * far_plane) / (near_plane - far_plane);
            result._data[3][3] = 0.0;
        }

        return result;
    }

    pub fn transpose(&self) -> Self {
        let mut result = Float4x4::default();

        unsafe {
            result._data[0][0] = self._data[0][0];
            result._data[0][1] = self._data[1][0];
            result._data[0][2] = self._data[2][0];
            result._data[0][3] = self._data[3][0];

            result._data[1][0] = self._data[0][1];
            result._data[1][1] = self._data[1][1];
            result._data[1][2] = self._data[2][1];
            result._data[1][3] = self._data[3][1];

            result._data[2][0] = self._data[0][2];
            result._data[2][1] = self._data[1][2];
            result._data[2][2] = self._data[2][2];
            result._data[2][3] = self._data[3][2];

            result._data[3][0] = self._data[0][3];
            result._data[3][1] = self._data[1][3];
            result._data[3][2] = self._data[2][3];
            result._data[3][3] = self._data[3][3];
        }

        return result;
    }

    pub fn translate_point(&self, point: Float4) -> Float4 {
        let mut result = Float4::zero();

        unsafe {
            let r0 = Float4{x: self._data[0][0], y: self._data[1][0], z: self._data[2][0], w: self._data[3][0] };
            let r1 = Float4{x: self._data[0][1], y: self._data[1][1], z: self._data[2][1], w: self._data[3][1] };
            let r2 = Float4{x: self._data[0][2], y: self._data[1][2], z: self._data[2][2], w: self._data[3][2] };
            let r3 = Float4{x: self._data[0][3], y: self._data[1][3], z: self._data[2][3], w: self._data[3][3] };

            result.x = point.dot(r0);
            result.y = point.dot(r1);
            result.z = point.dot(r2);
            result.w = point.dot(r3);
        }

        return result;
    }

    pub fn invert(&self) -> Self {
        let mut result = Float4x4::default();

        unsafe {
            let n11 = self._data[0][0]; let n12 = self._data[1][0]; let n13 = self._data[2][0]; let n14 = self._data[3][0];
            let n21 = self._data[0][1]; let n22 = self._data[1][1]; let n23 = self._data[2][1]; let n24 = self._data[3][1];
            let n31 = self._data[0][2]; let n32 = self._data[1][2]; let n33 = self._data[2][2]; let n34 = self._data[3][2];
            let n41 = self._data[0][3]; let n42 = self._data[1][3]; let n43 = self._data[2][3]; let n44 = self._data[3][3];

            let t11 = n23 * n34 * n42 - n24 * n33 * n42 + n24 * n32 * n43 - n22 * n34 * n43 - n23 * n32 * n44 + n22 * n33 * n44;
            let t12 = n14 * n33 * n42 - n13 * n34 * n42 - n14 * n32 * n43 + n12 * n34 * n43 + n13 * n32 * n44 - n12 * n33 * n44;
            let t13 = n13 * n24 * n42 - n14 * n23 * n42 + n14 * n22 * n43 - n12 * n24 * n43 - n13 * n22 * n44 + n12 * n23 * n44;
            let t14 = n14 * n23 * n32 - n13 * n24 * n32 - n14 * n22 * n33 + n12 * n24 * n33 + n13 * n22 * n34 - n12 * n23 * n34;

            let det = n11 * t11 + n21 * t12 + n31 * t13 + n41 * t14;
            assert!(!float_is_zero(det));

            let idet = 1.0 / det;

            result._data[0][0] = t11 * idet;
            result._data[0][1] = (n24 * n33 * n41 - n23 * n34 * n41 - n24 * n31 * n43 + n21 * n34 * n43 + n23 * n31 * n44 - n21 * n33 * n44) * idet;
            result._data[0][2] = (n22 * n34 * n41 - n24 * n32 * n41 + n24 * n31 * n42 - n21 * n34 * n42 - n22 * n31 * n44 + n21 * n32 * n44) * idet;
            result._data[0][3] = (n23 * n32 * n41 - n22 * n33 * n41 - n23 * n31 * n42 + n21 * n33 * n42 + n22 * n31 * n43 - n21 * n32 * n43) * idet;

            result._data[1][0] = t12 * idet;
            result._data[1][1] = (n13 * n34 * n41 - n14 * n33 * n41 + n14 * n31 * n43 - n11 * n34 * n43 - n13 * n31 * n44 + n11 * n33 * n44) * idet;
            result._data[1][2] = (n14 * n32 * n41 - n12 * n34 * n41 - n14 * n31 * n42 + n11 * n34 * n42 + n12 * n31 * n44 - n11 * n32 * n44) * idet;
            result._data[1][3] = (n12 * n33 * n41 - n13 * n32 * n41 + n13 * n31 * n42 - n11 * n33 * n42 - n12 * n31 * n43 + n11 * n32 * n43) * idet;

            result._data[2][0] = t13 * idet;
            result._data[2][1] = (n14 * n23 * n41 - n13 * n24 * n41 - n14 * n21 * n43 + n11 * n24 * n43 + n13 * n21 * n44 - n11 * n23 * n44) * idet;
            result._data[2][2] = (n12 * n24 * n41 - n14 * n22 * n41 + n14 * n21 * n42 - n11 * n24 * n42 - n12 * n21 * n44 + n11 * n22 * n44) * idet;
            result._data[2][3] = (n13 * n22 * n41 - n12 * n23 * n41 - n13 * n21 * n42 + n11 * n23 * n42 + n12 * n21 * n43 - n11 * n22 * n43) * idet;

            result._data[3][0] = t14 * idet;
            result._data[3][1] = (n13 * n24 * n31 - n14 * n23 * n31 + n14 * n21 * n33 - n11 * n24 * n33 - n13 * n21 * n34 + n11 * n23 * n34) * idet;
            result._data[3][2] = (n14 * n22 * n31 - n12 * n24 * n31 - n14 * n21 * n32 + n11 * n24 * n32 + n12 * n21 * n34 - n11 * n22 * n34) * idet;
            result._data[3][3] = (n12 * n23 * n31 - n13 * n22 * n31 + n13 * n21 * n32 - n11 * n23 * n32 - n12 * n21 * n33 + n11 * n22 * n33) * idet;
        }

        return result;
    }
}

pub fn mul_rh(left: Float4x4, right: Float4x4) -> Float4x4 {
    let mut result = Float4x4::default();

    unsafe {
        let lr0 = Float4::new(left._data[0][0], left._data[1][0], left._data[2][0], left._data[3][0]);
        let lr1 = Float4::new(left._data[0][1], left._data[1][1], left._data[2][1], left._data[3][1]);
        let lr2 = Float4::new(left._data[0][2], left._data[1][2], left._data[2][2], left._data[3][2]);
        let lr3 = Float4::new(left._data[0][3], left._data[1][3], left._data[2][3], left._data[3][3]);

        result._data[0][0] = lr0.dot(right._cols.c0);
        result._data[0][1] = lr1.dot(right._cols.c0);
        result._data[0][2] = lr2.dot(right._cols.c0);
        result._data[0][3] = lr3.dot(right._cols.c0);

        result._data[1][0] = lr0.dot(right._cols.c1);
        result._data[1][1] = lr1.dot(right._cols.c1);
        result._data[1][2] = lr2.dot(right._cols.c1);
        result._data[1][3] = lr3.dot(right._cols.c1);

        result._data[2][0] = lr0.dot(right._cols.c2);
        result._data[2][1] = lr1.dot(right._cols.c2);
        result._data[2][2] = lr2.dot(right._cols.c2);
        result._data[2][3] = lr3.dot(right._cols.c2);

        result._data[3][0] = lr0.dot(right._cols.c3);
        result._data[3][1] = lr1.dot(right._cols.c3);
        result._data[3][2] = lr2.dot(right._cols.c3);
        result._data[3][3] = lr3.dot(right._cols.c3);
    }

    return result;
}

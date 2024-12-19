use rand::prelude::*;

pub mod float2;
pub mod float3;
pub mod float4;
pub mod float4x4;

use float3::Float3;

// NOTE(enlynn): the purpose for this alias to lay the groundwork for testing different
//  types of Float. PBR-book allows for both f32 and FixedPoints and I might want to try
//  something similar in the future.
pub type Float = f32;

pub const FLOAT_INFINITY: Float = Float::INFINITY;

// Utility Functions
//

// --------------------------------------------------------------------
// NOTE(enlynn): This is a lazy attempt at addressing FP-error
//
// Source: https://isocpp.org/wiki/faq/newbie#floating-point-arith

// This function is not symmetrical - F32IsEqual(Left, Right) might not always equal F32IsEqual(Right, Left)
// TODO(enlynn): properly handle floating point error.
pub fn float_is_equal(left: f32, right: f32) -> bool {
    // TODO(enlynn):  max - min test to make sure there aren't perf implication...
    (left - right).abs() <= (Float::EPSILON * left.abs())
}

pub fn float_is_zero(v: f32) -> bool {
    return float_is_equal(v, 0.0);
}

// --------------------------------------------------------------------

pub fn degrees_to_radians(degrees: Float) -> Float {
    degrees * std::f32::consts::PI / 180.0
}

pub fn rand_float() -> Float {
    // TODO(enlynn): look into a high quality, fast rngs for Rust

    // Ideally this will fetch the local ThreadRng and only initialize on the first call.
    let mut rng = rand::thread_rng();
    rng.gen()
}

pub fn rand_float_in_range(min: Float, max: Float) -> Float {
    min + (max - min) * rand_float()
}

pub fn sample_in_square() -> Float3 {
    // Returns a vector to a random point in the [-0.5, -0.5] -> [0.5, 0.5] unit square
    Float3::new(rand_float() - 0.5, rand_float() - 0.5, 0.0)
}

// Finds a random vector pointing in the same direction as the normal.
pub fn rand_on_hemisphere(normal: &Float3) -> Float3 {
    //
    // cos(theta) = Normal . random_vector()
    // cos(theta) > 0  -> vector is in the same hemisphere
    // cos(theta) <= 0 -> vector is tangent or on the other hemisphere (points against the normal)
    let on_unit_sphere = Float3::rand_unit_vector();
    if on_unit_sphere.dot(*normal) > 0.0 {
        on_unit_sphere
    } else {
        -on_unit_sphere
    }
}

pub fn reflect(incoming_ray: &Float3, normal: &Float3) -> Float3 {
    // incoming_ray -> ray intersecting the surface
    // normal -> normal of the surface (assumed unit length)
    //
    // ir   n  or
    //  \   |   /|
    //   \  |  / | pr
    //    \ | /  |
    //   ---------
    //      \    |
    //       \   |
    //        \  |
    //      ir \ | pr
    //
    // ir - incoming_ray              (direction - downwards)
    // n  - normal                    (direction - upwards)
    // or - outgoing (reflected) ray  (direction - upwards)
    // pr - projected ray             (direction - upwards)
    //
    // Project the incoming Ray onto the normal vector:
    //   (incoming_ray . normal) * normal
    //
    // Using some quick maths:
    //   ir - 2*pr = or
    //   incoming_ray - 2 * ((incoming_ray . normal) * normal)
    *incoming_ray - (2.0 * incoming_ray.dot(*normal) * (*normal))
}

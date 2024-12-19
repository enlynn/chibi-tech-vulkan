
use std::path::PathBuf;

use common::math::float3::*;

pub struct ChibiMaterial {
    pub ambient_color: Float3,
    pub ambient_map:   PathBuf,
}

use super::material::*;

use common::math::{float3::*, float4::*};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Float3,
    pub uv_x:     f32,
    //----------------- 16-byte boundary
    pub normal:   Float3,
    pub uv_y:     f32,
    //----------------- 16-byte boundary
    pub color:    Float4,
    //----------------- 16-byte boundary
}

impl Vertex {
    pub fn new() -> Self {
        Self{
            position: Float3::zero(),
            uv_x:     0.0,
            normal:   Float3::zero(),
            uv_y:     0.0,
            color:    Float4::zero(),
        }
    }
}

pub struct ChibiImportGeometry {
    pub vertices:       Vec<Vertex>,
    pub indices:        Vec<u32>,
    pub material_index: Option<usize>,
}

pub struct ChibiImportMesh {
    pub geoms:     Vec<ChibiImportGeometry>,
    pub materials: Vec<ChibiImportMaterial>,
}

impl Default for Vertex {
    fn default() -> Self {
        Self{
            position: Float3::zero(),
            uv_x:     0.0,
            normal:   Float3::zero(),
            uv_y:     0.0,
            color:    Float4::zero(),
        }
    }
}

impl Default for ChibiImportGeometry {
    fn default() -> Self {
        Self{
            vertices:       Vec::new(),
            indices:        Vec::new(),
            material_index: None,
        }
    }
}

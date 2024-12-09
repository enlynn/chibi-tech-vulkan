use crate::math::{ float3::*, float4::*, float4x4::*};

use super::graphics::*;

use vendor::vulkan::*;

pub(crate) type MeshId = usize;

pub(crate) const MAX_LOADED_MESHES: usize = 100;

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

#[derive(Clone, Copy)]
pub(crate) struct GpuMeshBuffers {
    pub index_buffer:          AllocatedBuffer,
    pub vertex_buffer:         AllocatedBuffer,
    pub vertex_buffer_address: VkDeviceAddress,
    pub index_count:           u32,
    pub transform:             Float4x4,
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

impl Default for GpuMeshBuffers {
    fn default() -> Self {
        Self{
            index_buffer:          AllocatedBuffer::default(),
            vertex_buffer:         AllocatedBuffer::default(),
            vertex_buffer_address: 0,
            index_count:           0,
            transform:             Float4x4::identity(),
        }
    }
}

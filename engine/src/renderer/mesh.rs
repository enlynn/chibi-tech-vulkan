use common::math::{ float3::*, float4::*, float4x4::*};

use common::util::id::*;
use assetlib::mesh::Vertex;

use super::graphics::{*, gpu_command_buffer::*, gpu_device::* };
use super::material::*;
use super::buffer::*;
use super::shader::*;

use std::fmt::Pointer;
use std::rc::Rc;

use vendor::vulkan::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeshId(Id);

pub(crate) const INVALID_MESH_ID:   MeshId = MeshId(INVALID_ID);
pub(crate) const MAX_LOADED_MESHES: usize  = 500;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub(crate) enum MeshFlags {
    None  = 0x00,
    Alive = 0x01,
}

#[derive(Clone, Copy)]
pub(crate) struct MeshMetadata {
    id:        MeshId,
    ref_count: usize,
    flags:     MeshFlags,
}

#[derive(Clone, Copy)]
pub(crate) struct GpuMeshBuffers {
    pub material:              MaterialInstanceId,
    pub index_buffer:          AllocatedBuffer,
    pub vertex_buffer:         AllocatedBuffer,
    pub vertex_buffer_address: VkDeviceAddress,
    pub index_count:           u32,
    pub transform:             Float4x4,
}

pub(crate) struct GpuMeshDraw {
    pub index_buffer:  AllocatedBuffer,
    pub index_count:   u32,
    pub push_constant: GpuDrawPushConstants,
    pub material:      MaterialInstanceId,
}

impl Default for GpuMeshBuffers {
    fn default() -> Self {
        Self{
            index_buffer:          AllocatedBuffer::default(),
            vertex_buffer:         AllocatedBuffer::default(),
            vertex_buffer_address: 0,
            index_count:           0,
            transform:             Float4x4::identity(),
            material:              INVALID_MATERIAL_INSTACE_ID,
        }
    }
}

impl Default for MeshMetadata {
    fn default() -> Self {
        Self{
            id:        INVALID_MESH_ID,
            ref_count: 0,
            flags:     MeshFlags::None,
        }
    }
}

pub(crate) struct MeshCreateInfo {
    pub vertices:     *const Vertex,
    pub vertex_count: usize,

    pub indices:      *const u32,
    pub index_count:  usize,

    //todo: other mesh properties
    pub transform:    Float4x4,
    pub material:     MaterialInstanceId,
}

pub(crate) struct MeshSystem {
    device:   Rc<Device>,
    metadata: [MeshMetadata;   MAX_LOADED_MESHES],
    meshes:   [GpuMeshBuffers; MAX_LOADED_MESHES],
    id_gen:   IdSystem,
    gpu_data: StructuredBuffer<GpuMeshUniform>,
}

impl MeshSystem {
    pub fn new(device: Rc<Device>, buffered_frames: usize) -> Self {
        let gpu_data: StructuredBuffer<GpuMeshUniform> = StructuredBuffer::new(&device, MAX_LOADED_MESHES, buffered_frames);

        return Self{
            device,
            metadata: [MeshMetadata::default();   MAX_LOADED_MESHES],
            meshes:   [GpuMeshBuffers::default(); MAX_LOADED_MESHES],
            id_gen:   IdSystem::new(MAX_LOADED_MESHES),
            gpu_data
        }
    }

    pub fn destroy(&mut self) {
        for i in 0..MAX_LOADED_MESHES {
            let meta = &self.metadata[i];
            if self.id_gen.is_id_valid(meta.id.0) {
                self.device.destroy_buffer(&mut self.meshes[i].vertex_buffer);
                self.device.destroy_buffer(&mut self.meshes[i].index_buffer);
                //todo: perhaps release the reference to the material?
            }
        }

        self.gpu_data.destroy(&self.device);
    }

    pub fn create_mesh(&mut self, info: MeshCreateInfo, mut command_buffer: &mut CommandBuffer, fence: VkFence) -> MeshId {
        let result: MeshId = MeshId(self.id_gen.alloc_id().unwrap_or(INVALID_MESH_ID.0));
        if result == INVALID_MESH_ID {
            return result;
        }

        let vertex_buffer_size = info.vertex_count * std::mem::size_of::<Vertex>();
        let index_buffer_size  = info.index_count  * std::mem::size_of::<u32>();

        let vertex_buffer_flags = VK_BUFFER_USAGE_STORAGE_BUFFER_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_SHADER_DEVICE_ADDRESS_BIT;
        let index_buffer_flags  = VK_BUFFER_USAGE_INDEX_BUFFER_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT;

        let mut buffer = &mut self.meshes[result.0.get_index() as usize];
        buffer.index_buffer          = self.device.create_buffer(index_buffer_size, index_buffer_flags, VMA_MEMORY_USAGE_GPU_ONLY);
        buffer.vertex_buffer         = self.device.create_buffer(vertex_buffer_size, vertex_buffer_flags, VMA_MEMORY_USAGE_GPU_ONLY);
        buffer.vertex_buffer_address = self.device.get_buffer_device_address(&buffer.vertex_buffer);
        buffer.index_count           = info.index_count as u32;
        buffer.transform             = info.transform;
        buffer.material              = info.material;

       	let mut staging_buffer = self.device.create_buffer(vertex_buffer_size + index_buffer_size, VK_BUFFER_USAGE_TRANSFER_SRC_BIT, VMA_MEMORY_USAGE_CPU_ONLY);

        let mut memory = staging_buffer.info.pMappedData;
        assert!(memory != std::ptr::null_mut());

        // Copy vertex data to the staging buffer
        let mut memory_as_vertex = memory as *mut Vertex;
        unsafe { std::ptr::copy(info.vertices, memory_as_vertex, info.vertex_count) };

        // Copy index data to the staging buffer
        let mut memory_as_index = unsafe { memory.add(vertex_buffer_size) } as *mut u32;
        unsafe { std::ptr::copy(info.indices, memory_as_index, info.index_count) };

        super::util::immediate_submit(&self.device, &mut command_buffer, fence,
            |command_buffer: &CommandBuffer| {
                // Copy to the final vertex buffer
                command_buffer.copy_buffer(&buffer.vertex_buffer, 0, &staging_buffer, 0, vertex_buffer_size as VkDeviceSize);
                // Copy to the final index buffer
                command_buffer.copy_buffer(&buffer.index_buffer, 0, &staging_buffer, vertex_buffer_size as VkDeviceSize, index_buffer_size as VkDeviceSize);
            }
        );

        self.device.destroy_buffer(&mut staging_buffer);

        let mut meta = &mut self.metadata[result.0.get_index() as usize];
        meta.id    = result;
        meta.flags = MeshFlags::Alive;

        return result;
    }

    pub fn destroy_mesh(&mut self, mesh_id: MeshId) {
        if self.id_gen.is_id_valid(mesh_id.0) && self.metadata[mesh_id.0.get_index() as usize].ref_count == 1 {
            let mut mesh = &mut self.meshes[mesh_id.0.get_index() as usize];

            self.device.destroy_buffer(&mut mesh.vertex_buffer);
            self.device.destroy_buffer(&mut mesh.index_buffer);
            //todo: perhaps release the reference to the material?
            self.id_gen.free_id(mesh_id.0);
        }
    }

    pub fn collect_live_meshes(&self) -> Vec<GpuMeshBuffers> {
        //todo: look into using a cache to speed this up
        let mut result = Vec::with_capacity(MAX_LOADED_MESHES);

        for i in 0..MAX_LOADED_MESHES {
            let meta = &self.metadata[i];
            if self.id_gen.is_id_valid(meta.id.0) && (meta.flags as u32 & MeshFlags::Alive as u32 != 0) {
                result.push(self.meshes[i]);
            }
        }

        return result;
    }

    pub fn build_draw_list(&mut self, frame_index: usize, meshes_to_draw: &[GpuMeshBuffers]) -> Vec<GpuMeshDraw> {
        //todo: this function can be parallelized quite a bit.

        let mut draws: Vec<GpuMeshDraw> = Vec::with_capacity(meshes_to_draw.len());

        self.gpu_data.map_frame(frame_index);
        let uniform_buffer_address = self.gpu_data.get_device_address();

        for mesh in meshes_to_draw {
            let uniform = GpuMeshUniform{
                transform: mesh.transform,
            };

            let index = self.gpu_data.write_next(uniform);

            let push_const = GpuDrawPushConstants{
                vertex_buffer:    mesh.vertex_buffer_address,
               	mesh_data_buffer: uniform_buffer_address,
               	//----------------- 16-byte boundary
               	//material_buffer:  VkDeviceAddress,
               	mesh_index:       index as u32,
            };

            let draw = GpuMeshDraw{
                index_buffer:  mesh.index_buffer,
                index_count:   mesh.index_count,
                push_constant: push_const,
                material:      mesh.material,
            };

            draws.push(draw);
        }

        self.gpu_data.unmap_frame();

        return draws;
    }
}

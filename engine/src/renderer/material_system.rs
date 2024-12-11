use vendor::vulkan::*;

use super::graphics::{
    *,
    gpu_device::*,
    gpu_swapchain::*,
    gpu_utils::*,
    gpu_command_pool::*,
    gpu_command_buffer::*,
    gpu_descriptors::*,
    gpu_pipeline::*,
    gpu_descriptors::*,
};

use crate::util::id::*;
use crate::math::float4::*;

use std::rc::Rc;

struct MaterialId(Id);
struct MaterialInstanceId(Id);
struct TextureId(Id);

struct Texture2D {
    image:   AllocatedImage,
    sampler: VkSampler,
}

struct MaterialResources {
    color_image:        TextureId,
}

struct MaterialInstance {
    set:                VkDescriptorSet,
    resources:          MaterialResources,
    data_buffer:        VkBuffer,
	data_buffer_offset: u32,
}

// struct Material {
//     pipeline:        VkPipeline,
//     layout:          VkPipelineLayout,
//     instance_stride: usize,
//     instances:       Vec<MaterialInstance>,
//     instance_ids:    IdSystem,
// }

struct MaterialSystemCreateInfo {
    max_textures:        usize,
    max_materials:       usize,
    max_material_memory: usize,
}

pub const SHADER_BIND_POINT_SCENE:         u32 = 0;
pub const SHADER_BIND_POINT_TEXTURES:      u32 = 1;
pub const SHADER_BIND_POINT_MAT_INSTANCES: u32 = 2;

trait Material {
    fn on_init(&mut self, device: &Device);
    fn on_destroy(&mut self, device: &Device);
    fn on_bind(&mut self, device: &Device);

    fn alloc_instance(&mut self, device: &Device) -> MaterialInstanceId;
    fn free_instance(&mut self, device: &Device, instance_id: MaterialInstanceId);
}

struct TriangleResources {
    color_texture: TextureId,
}

struct TriangleInstanceData {
    ambient_color: Float4,
}

struct TriangleMaterial {
    pipeline:          VkPipeline,
    pipeline_layout:   VkPipelineLayout,
    descriptor_layout: VkDescriptorSetLayout,

    // Scene data
    //

    // taken per-frame from FrameState (?)
    scene_data:        VkDescriptorSet,

    // Per instance data
    //

    // Descriptor Set: Textures
    resources: Vec<TriangleResources>,
    instances: Vec<TriangleInstanceData>,

    // Data
    // ambient_color: Float4,
}

/*

-> material_buffer:     AllocatedBuffer,   per-frame material buffer
-> meshes_to_draw:      &[Mesh],           mesh draws this frame for the material
-> mesh_push_constants: &mut [PushConsts], mesh push constants. can set the dynamic material offset in them

struct MaterialRenderPacket {
    pipeline: VkPipeline,       // pipeline for the material
    layout:   VkPipelineLayout, // pipeline layout for the material

    buffer_offset: u32,         // offset into the dynamic material buffer
    buffer_size:   u32,         // size of the material used
}

sort(draws, by_material)
foreach material:
    sort(draws[mat.start..mat.end], by_material_instance)

foreach draw
    make_push_constant(push_data)

foreach material
    mat.update(&draws[mat.start..mat.end], &mut push_data[mat.start..mat.end], &mut material_instance_buffer)

foreach material
    mat.bind()
    draw(&draws[mat.start..mat.end])

*/

impl Material for TriangleMaterial {
    fn on_init(&mut self, device: &Device) {

    }

    fn on_destroy(&mut self, device: &Device) {

    }

    fn on_bind(&mut self, device: &Device) {

    }

    fn alloc_instance(&mut self, device: &Device) -> MaterialInstanceId {
        todo!()
    }

    fn free_instance(&mut self, device: &Device, instance_id: MaterialInstanceId) {
        todo!()
    }
}

enum MaterialBindType {
    UniformBuffer(AllocatedBuffer),   // VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER or VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC
    Image,                            // VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER
    Instance,                         // VK_DESCRIPTOR_TYPE_STORAGE_BUFFER or VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC
}

struct MaterialShaderBinding {
    bind_point: u32,
    bind_type:  MaterialBindType,
}

struct MaterialCreateInfo {
    max_instances:   usize,
    instance_stride: usize,
    //todo: pipeline info
    //todo: pipeline layout info
}

struct MaterialInstanceCreateInfo {

}

struct MaterialDataAllocation {
    buffer: AllocatedBuffer,
}

struct MaterialDataBuffer {
    buffer:      AllocatedBuffer,
    address:     VkDeviceAddress,
    next_offset: usize,
    //todo: maybe mapped pointer?
}

struct MaterialSystem {
    device:               Rc<Device>,
    descriptor_allocator: DescriptorAllocatorGrowable,
    textures:             Vec<Texture2D>,
    texture_ids:          IdSystem,
    materials:            Vec<Box<dyn Material>>,
    material_ids:         IdSystem,

    //todo: example textures.
}

impl MaterialDataBuffer {
    pub fn new(device: &Device, max_size: usize) -> Self {
        let buffer_flags = VK_BUFFER_USAGE_STORAGE_BUFFER_BIT | VK_BUFFER_USAGE_SHADER_DEVICE_ADDRESS_BIT;
        let memory_flags = VMA_MEMORY_USAGE_CPU_TO_GPU;

        //todo: are there any alignment concerns?
        let buffer  = device.create_buffer(max_size, buffer_flags, memory_flags);
        let address = device.get_buffer_device_address(&buffer);

        return Self{ buffer, address, next_offset: 0 };
    }

    pub fn destroy(&mut self, device: &Device) {
        device.destroy_buffer(&mut self.buffer);
    }

    pub fn alloc(&mut self, size: usize) -> MaterialDataAllocation {
        assert!(self.next_offset + size <= self.buffer.info.size as usize);

        let mut allocation = self.buffer;
        allocation.info.offset += self.next_offset as u64;

        self.next_offset += size;

        return MaterialDataAllocation{
            buffer: allocation,
        };
    }
}

impl MaterialSystem {
    pub fn new(device: Rc<Device>, info: MaterialSystemCreateInfo) -> MaterialSystem {
        // arbritrarily chosen - todo: fine tune
        let ratios: [PoolSizeRatio; 4] = [
            PoolSizeRatio{descriptor_type: VK_DESCRIPTOR_TYPE_STORAGE_IMAGE,          ratio: 3.0 },
            PoolSizeRatio{descriptor_type: VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,         ratio: 3.0 },
            PoolSizeRatio{descriptor_type: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,         ratio: 3.0 },
            PoolSizeRatio{descriptor_type: VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER, ratio: 4.0 },
        ];

        let descriptor_allocator = DescriptorAllocatorGrowable::new(&device, &ratios, 1000);

        Self{
            device,
            descriptor_allocator,
            textures:     Vec::new(),
            texture_ids:  IdSystem::new(info.max_textures),
            materials:    Vec::new(),
            material_ids: IdSystem::new(info.max_materials),
        }
    }

    pub fn register_material(&mut self, material: Box<dyn Material>) -> MaterialId {
        todo!()
    }

    pub fn alloc_texture(&mut self) -> TextureId {
        todo!()
    }

    pub fn free_texture(&mut self, texture_id: TextureId) {
        todo!()
    }

    pub fn alloc_material(&mut self, info: MaterialCreateInfo) -> MaterialId {
        todo!()
    }

    pub fn free_material(&mut self, material_id: MaterialId) {
        todo!()
    }

    pub fn alloc_material_instance<T>(&mut self, material_id: MaterialId, rsrc: MaterialResources, data: T) -> MaterialInstanceId {
        todo!()
    }

    pub fn free_material_instance(&mut self, material_id: MaterialId, instance_id: MaterialInstanceId) {
        todo!()
    }
}

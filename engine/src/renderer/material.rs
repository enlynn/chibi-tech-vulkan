use vendor::vulkan::*;
use common::util::id::*;

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

use common::math::float4::*;

use super::texture::*;
use super::shader::*;

use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialId(Id);
pub const INVALID_MATERIAL_ID: MaterialId = MaterialId(INVALID_ID);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MaterialInstanceId(MaterialId, Id);
pub const INVALID_MATERIAL_INSTACE_ID: MaterialInstanceId = MaterialInstanceId(INVALID_MATERIAL_ID, INVALID_ID);

//                                                        Index         Generation
pub(crate) const OPAQUE_MAT_ID: MaterialId = MaterialId(Id(0 as IdType | ((0 as IdType) << IDX_BITS) as IdType));

struct MaterialSystemCreateInfo {
    max_textures:        usize,
    max_materials:       usize,
    max_material_memory: usize,
}

pub const SHADER_BIND_POINT_SCENE:         u32 = 0;
pub const SHADER_BIND_POINT_TEXTURES:      u32 = 1;
pub const SHADER_BIND_POINT_MAT_INSTANCES: u32 = 2;

pub(crate) trait Material {
    fn on_init(&mut self, device: &Device, buffered_frames: usize);
    fn on_destroy(&mut self, device: &Device);
    fn on_bind(&mut self, device: &Device);

    fn alloc_instance(&mut self, device: &Device) -> MaterialInstanceId;
    fn free_instance(&mut self, device: &Device, instance_id: MaterialInstanceId);
}

pub(crate) struct OpaqueResources {
    color_texture:      TextureId,
    data_buffer:        VkBuffer,
	data_buffer_offset: u32,
}

#[repr(C)]
pub(crate) struct OpaqueInstanceData {
    ambient_color: Float4,
}

const MAX_OPAQUE_INSTANCES: usize = 1000;
pub(crate) struct OpaqueMaterial {
    pub(crate) pipeline:            VkPipeline,
    pub(crate) pipeline_layout:     VkPipelineLayout,
    pub(crate) scene_dl:            VkDescriptorSetLayout,
    pub(crate) texture_resource_dl: VkDescriptorSetLayout,

    // Descriptor Set: Textures
    id_gen:            IdSystem,
    resources:         Vec<OpaqueResources>,
    instances:         Vec<OpaqueInstanceData>,
    gpu_buffer:        Vec<AllocatedBuffer>,
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

impl Default for OpaqueMaterial {
    fn default() -> Self {
        Self{
            pipeline:            std::ptr::null_mut(),
            pipeline_layout:     std::ptr::null_mut(),
            scene_dl:            std::ptr::null_mut(),
            texture_resource_dl: std::ptr::null_mut(),
            id_gen:              IdSystem::new(MAX_OPAQUE_INSTANCES),
            resources:           Vec::with_capacity(MAX_OPAQUE_INSTANCES),
            instances:           Vec::with_capacity(MAX_OPAQUE_INSTANCES),
            gpu_buffer:          Vec::new(),
        }
    }
}

impl Material for OpaqueMaterial {
    fn on_init(&mut self, device: &Device, buffered_frames: usize) {
        // Create pipeline structures
        //

        let colored_tri_vert_sm = load_shader_module(&device, "colored_triangle", ShaderStage::Vertex);
        let colored_tri_frag_sm = load_shader_module(&device, "colored_triangle", ShaderStage::Fragment);

        let global_dl = {
            let mut builder = DescriptorLayoutBuilder::new();
            builder.add_binding(0, VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER);
            builder.build(&device, VK_SHADER_STAGE_VERTEX_BIT, 0)
        };

        let descriptor_layout = {
            let mut builder = DescriptorLayoutBuilder::new();
            builder.add_binding(0, VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER);
            builder.build(&device, VK_SHADER_STAGE_FRAGMENT_BIT, 0)
        };

        let pipeline_layout = {
            let descriptors:    [VkDescriptorSetLayout; 2] = [ global_dl, descriptor_layout ];
            let push_constants: [VkPushConstantRange;   1] = [
                VkPushConstantRange{
                    stageFlags: VK_SHADER_STAGE_VERTEX_BIT,
                    offset:     0,
                    size:       std::mem::size_of::<GpuDrawPushConstants>() as u32,
                },
            ];

            device.create_pipeline_layout(descriptors.as_slice(), push_constants.as_slice())
        };

        let pipeline = {
            let mut builder = GraphicsPipelineBuilder::new();

            //use the triangle layout we created
            builder
                .set_pipeline_layout(pipeline_layout)
            //connecting the vertex and pixel shaders to the pipeline
                .set_shaders(colored_tri_vert_sm, colored_tri_frag_sm)
            //it will draw triangles
                .set_input_topology(VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST)
            //filled triangles
                .set_polygon_mode(VK_POLYGON_MODE_FILL)
            //no backface culling
                .set_cull_mode(VK_CULL_MODE_BACK_BIT, VK_FRONT_FACE_CLOCKWISE)
            //no multisampling
                .set_multisampling_none()
            //no blending
                .disable_blending()
            // additive blending
                //.enabled_blending_additive()
            // alpha blending
                //.enabled_blending_alphablend()
            //no depth testing
                //.disable_depth_test()
            // enabled depth testing
                .enable_depth_test(true, VK_COMPARE_OP_LESS_OR_EQUAL)
            //connect the image format we will draw into, from draw image
                .set_color_attachment_format(VK_FORMAT_R16G16B16A16_SFLOAT)
                .set_depth_format(device.get_depth_format());

            //finally build the pipeline
            builder.build(&device)
        };

        device.destroy_shader_module(colored_tri_vert_sm);
        device.destroy_shader_module(colored_tri_frag_sm);

        // Create buffered per-frame material data.
        //

        let allocation_size: usize              = MAX_OPAQUE_INSTANCES * std::mem::size_of::<OpaqueInstanceData>();
        let buffer_usage:    VkBufferUsageFlags = VK_BUFFER_USAGE_STORAGE_BUFFER_BIT | VK_BUFFER_USAGE_SHADER_DEVICE_ADDRESS_BIT;
        let memory_usage:    VmaMemoryUsage     = VMA_MEMORY_USAGE_CPU_TO_GPU;

        self.gpu_buffer = Vec::with_capacity(buffered_frames);
        for buffer in &mut self.gpu_buffer {
            *buffer = device.create_buffer(allocation_size, buffer_usage, memory_usage);
        }

        self.pipeline            = pipeline;
        self.pipeline_layout     = pipeline_layout;
        self.scene_dl            = global_dl;
        self.texture_resource_dl = descriptor_layout;
    }

    fn on_destroy(&mut self, device: &Device) {
        device.destroy_pipeline(self.pipeline);
        device.destroy_pipeline_layout(self.pipeline_layout);

        for mut buffer in &mut self.gpu_buffer {
            device.destroy_buffer(&mut buffer);
        }
    }

    fn on_bind(&mut self, device: &Device, ) {
        todo!()
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
}

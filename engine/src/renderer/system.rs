use std::borrow::BorrowMut;
use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;
use std::str::FromStr;
use std::collections::VecDeque;

use common::math::{ float3::*, float4::*, float4x4::* };
use crate::window::NativeSurface;
use crate::util::ffi::*;

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

use super::command_buffer::*;
use super::mesh::*;
use super::shader::*;
use super::texture::*;
use super::material::*;

use vendor::vulkan::*;

struct PerFrameCommandBuffer {
    pool:   CommandPool,
    handle: CommandBuffer,
}

struct PerFrameDeletionQueues {
    buffer_deletion_queue: VecDeque<AllocatedBuffer>,
    image_deletion_queue:  VecDeque<AllocatedImage>,
}

struct SceneData {
    gpu_buffer: AllocatedBuffer,
    scene:      GlobalSceneData,
}

struct PerFrameState {
    frame_index: usize,
    scene_data:  SceneData,
}

struct PerFrameData {
    command_buffer:      RefCell<PerFrameCommandBuffer>, // i don't like this one bit...
    dynamic_descriptors: RefCell<DescriptorAllocatorGrowable>,
    deletion_queues:     RefCell<PerFrameDeletionQueues>,
    state:               RefCell<PerFrameState>,
}

struct ComputeEffect {
    pub name:      String,
	pub pipeline:  VkPipeline,
	pub layout:    VkPipelineLayout,
	pub push_data: ComputePushConstants,
}

pub struct RendererCreateInfo {
    pub surface: NativeSurface,
}

pub struct RenderSystem{
    device:      Rc<Device>,
    swapchain:   Swapchain,
    scene_image: AllocatedImage,
    depth_image: AllocatedImage,

    // todo: look into using get_mut() when there is a single reference to the Rc. This might
    //       allow me to avoid using RefCell.
    frame_data:  Vec<Rc<PerFrameData>>,
    frame_index: usize,

    global_da:     DescriptorAllocator,
    draw_image_dl: VkDescriptorSetLayout,
    draw_image_ds: VkDescriptorSet,

    scene_data:      GlobalSceneData,
    global_scene_dl: VkDescriptorSetLayout,

    // immediate context submission - not quite sure where to put this right now
    //   This is largely used for an Upload Context for pushing data to the GPU.
    //   Ideally, I would be using some sort of paged heap to push data instead
    //   of waiting for every single mesh to upload before moving on.
    imm_fence:                Fence,
    imm_command_buffer:       CommandBuffer,
    imm_command_pool:         CommandPool,

	// for the background
	gradient_pl:              VkPipelineLayout,
	gradient_p:               VkPipeline,

	compute_effects:          Vec<ComputeEffect>,
	current_compute_effect:   usize,

	// Mesh "System"
	mesh_system:              MeshSystem,

	// Texture "System"
	texture_system:           TextureSystem,
	white_image:              TextureId,
	black_image:              TextureId,
	grey_image:               TextureId,
	error_checkerboard_image: TextureId,

	// Material "System"
	opaque_material:          OpaqueMaterial,
	default_material:         MaterialInstanceId,

	// Camera data
	view_matrix:              Float4x4,
	perspective_matrix:       Float4x4,

	// Outgoing Commands to the engine
	//   Will probably want this as a mpsc::Sender once the Renderer gets put on its own thread.
	outgoing_commands:        RenderCommandBuffer,
}

impl RenderSystem {
    fn create_scene_images(device: &Device, extent: VkExtent3D) -> AllocatedImage {
        let image_usages: VkImageUsageFlags =
            VK_IMAGE_USAGE_TRANSFER_SRC_BIT |
            VK_IMAGE_USAGE_TRANSFER_DST_BIT |
            VK_IMAGE_USAGE_STORAGE_BIT      |
            VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT;

        device.allocate_image_memory(
            extent,
            VK_FORMAT_R16G16B16A16_SFLOAT,
            image_usages,
            VMA_MEMORY_USAGE_GPU_ONLY,
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            false,
        )
    }

    fn create_depth_image(device: &Device, extent: VkExtent3D) -> AllocatedImage {
        let depth_format = device.get_depth_format();
        let image_usages = VK_IMAGE_USAGE_DEPTH_STENCIL_ATTACHMENT_BIT;

        device.allocate_image_memory(
            extent,
            depth_format,
            image_usages,
            VMA_MEMORY_USAGE_GPU_ONLY,
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            false,
        )
    }

    fn resize_device_resources(&mut self) {
        self.device.wait_idle();

        self.swapchain = self.device.create_swapchain(Some(&self.swapchain));
        self.swapchain.validate();

        self.device.destroy_image_memory(&mut self.scene_image);
        self.device.destroy_image_memory(&mut self.depth_image);

        self.scene_image = RenderSystem::create_scene_images(&self.device, self.swapchain.get_extent());
        self.depth_image = RenderSystem::create_depth_image(&self.device,  self.swapchain.get_extent());

        self.device.clear_descriptor_allocator(&self.global_da);
        self.draw_image_ds = {
            let ds = self.device.allocate_descriptors(&self.global_da, self.draw_image_dl).expect("Failed to alloc descriptor set");

            let mut writer = DescriptorWriter::new();
            writer.write_storage_image(0, self.scene_image.view, VK_IMAGE_LAYOUT_GENERAL);
            writer.update_set(&self.device, ds);

            ds
        };
    }

    pub fn new(create_info: RendererCreateInfo) -> RenderSystem {
        let device = Rc::new(Device::new(gpu_device::CreateInfo{
            features:         gpu_device::Features::default(),  //todo: make configurable
            surface:          create_info.surface,
            software_version: crate::make_app_version(0, 0, 1), //todo: make configurable
            software_name:    String::from("Testbed"),          //todo: make configurable
        }));

        let swapchain = device.create_swapchain(None);

        let scene_image = RenderSystem::create_scene_images(&device, swapchain.get_extent());
        let depth_image = RenderSystem::create_depth_image(&device,  swapchain.get_extent());

        let init_frame_data = |device: &Device| -> PerFrameData {
            let pool =   device.create_command_pool(QueueType::Graphics);
            let buffer = device.create_command_buffer(&pool);

            let sizes: [PoolSizeRatio; 4] = [
                PoolSizeRatio{descriptor_type: VK_DESCRIPTOR_TYPE_STORAGE_IMAGE,          ratio: 3.0 },
                PoolSizeRatio{descriptor_type: VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,         ratio: 3.0 },
                PoolSizeRatio{descriptor_type: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,         ratio: 3.0 },
                PoolSizeRatio{descriptor_type: VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER, ratio: 4.0 },
            ];

            PerFrameData{
                command_buffer:      RefCell::new(PerFrameCommandBuffer {pool, handle: buffer}),
                dynamic_descriptors: RefCell::new(DescriptorAllocatorGrowable::new(device, &sizes, 1000)),
                deletion_queues:     RefCell::new(PerFrameDeletionQueues{
                    buffer_deletion_queue: VecDeque::new(),
                    image_deletion_queue:  VecDeque::new(),
                }),
                state:               RefCell::new(PerFrameState{
                    frame_index: 0,
                    scene_data:  SceneData {
                        gpu_buffer: device.create_buffer(std::mem::size_of::<GlobalSceneData>(), VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT, VMA_MEMORY_USAGE_CPU_TO_GPU),
                        scene:      GlobalSceneData{
                            view:           Float4x4::identity(),
                            proj:           Float4x4::identity(),
                            view_proj:      Float4x4::identity(),
                            ambient_color:  Float4::zero(),
                            sunlight_dir:   Float4::zero(),
                            sunlight_color: Float4::zero(),
                            padding0:       Float4::zero(),
                        },
                    },
                }),
            }
        };

        let mut frame_data = Vec::<Rc<PerFrameData>>::with_capacity(swapchain.images.len());
        for i in 0..swapchain.images.len() {
            frame_data.push(Rc::new(init_frame_data(&device)));
        }

        // Create immediate submission context
        //

        let imm_fence:          VkFence       = device.create_fence(true);
        let imm_command_pool:   CommandPool   = device.create_command_pool(QueueType::Graphics);
        let imm_command_buffer: CommandBuffer = device.create_command_buffer(&imm_command_pool);

        // Create descriptors
        //

        let global_da = {
            let sizes: [PoolSizeRatio; 1] = [
                PoolSizeRatio{descriptor_type: VK_DESCRIPTOR_TYPE_STORAGE_IMAGE, ratio: 1.0 },
            ];

            device.create_descriptor_allocator(10, DescriptorAllocatorFlags::None, sizes.as_slice())
        };

        let draw_image_dl = {
            let mut builder = DescriptorLayoutBuilder::new();
            builder.add_binding(0, VK_DESCRIPTOR_TYPE_STORAGE_IMAGE);

            builder.build(&device, VK_SHADER_STAGE_COMPUTE_BIT, 0)
        };

        let draw_image_ds = {
            let ds = device.allocate_descriptors(&global_da, draw_image_dl).expect("Failed to alloc descriptor set");

            let mut writer = DescriptorWriter::new();
            writer.write_storage_image(0, scene_image.view, VK_IMAGE_LAYOUT_GENERAL);
            writer.update_set(&device, ds);

            ds
        };

        let gpu_global_scene_dl = {
            let mut build = DescriptorLayoutBuilder::new();
            build.add_binding(0, VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER);
            build.build(&device, VK_SHADER_STAGE_VERTEX_BIT | VK_SHADER_STAGE_FRAGMENT_BIT, 0)
        };

        // The Compute Pipeline
        //

        let gradient_sm = load_shader_module(&device, "gradient", ShaderStage::Compute);

        let gradient_pl = {
            let descriptors:    [VkDescriptorSetLayout; 1] = [ draw_image_dl ];
            let push_constants: [VkPushConstantRange; 0]   = [];

            device.create_pipeline_layout(descriptors.as_slice(), push_constants.as_slice())
        };

        let gradient_p = device.create_compute_pipeline(gradient_sm, gradient_pl);

        device.destroy_shader_module(gradient_sm);

        // Gradient Color Compute Effect
        //

        let compute_effect_gradient = {
            let gradient_color_sm = load_shader_module(&device, "gradient_color", ShaderStage::Compute);

            let gradient_color_pl = {
                let descriptors:    [VkDescriptorSetLayout; 1] = [ draw_image_dl ];
                let push_constants: [VkPushConstantRange; 1]   = [
                    make_push_constant_range(0, std::mem::size_of::<ComputePushConstants>() as u32, VK_SHADER_STAGE_COMPUTE_BIT),
                ];

                device.create_pipeline_layout(descriptors.as_slice(), push_constants.as_slice())
            };

            let gradient_color_p = device.create_compute_pipeline(gradient_color_sm, gradient_color_pl);

            device.destroy_shader_module(gradient_color_sm);

            ComputeEffect{
                name:      String::from("Gradient Effect"),
               	pipeline:  gradient_color_p,
               	layout:    gradient_color_pl,
               	push_data: ComputePushConstants{
                    data1: Float4::new(1.0, 0.0, 0.0, 1.0),
                    data2: Float4::new(0.0, 0.0, 1.0, 1.0),
                    data3: Float4::zero(),
                    data4: Float4::zero(),
                },
            }
        };

        // Sky Compute Effect
        //

        let sky_effect = {
            let sky_sm = load_shader_module(&device, "sky", ShaderStage::Compute);

            let sky_pl = {
                let descriptors:    [VkDescriptorSetLayout; 1] = [ draw_image_dl ];
                let push_constants: [VkPushConstantRange; 1]   = [
                    make_push_constant_range(0, std::mem::size_of::<ComputePushConstants>() as u32, VK_SHADER_STAGE_COMPUTE_BIT),
                ];

                device.create_pipeline_layout(descriptors.as_slice(), push_constants.as_slice())
            };

            let sky_p = device.create_compute_pipeline(sky_sm, sky_pl);

            device.destroy_shader_module(sky_sm);

            ComputeEffect{
                name:      String::from("Sky"),
               	pipeline:  sky_p,
               	layout:    sky_pl,
               	push_data: ComputePushConstants{
                    data1: Float4::new(0.1, 0.2, 0.4 ,0.97),
                    data2: Float4::zero(),
                    data3: Float4::zero(),
                    data4: Float4::zero(),
                },
            }
        };

        // Colored Triangle Pipeline
        //

        let mut opaque_material = OpaqueMaterial::default();
        opaque_material.on_init(&device, swapchain.get_image_count());

        // Some Default samplers
        //

        let texture_system = TextureSystem::new(device.clone());
        let mesh_system    = MeshSystem::new(device.clone(), swapchain.get_image_count());

        let mut result = RenderSystem{
            device,
            swapchain,
            scene_image,
            depth_image,
            frame_data,
            frame_index: 0,
            global_da,
            draw_image_dl,
            draw_image_ds,
            scene_data:      GlobalSceneData::default(),
            global_scene_dl: gpu_global_scene_dl,
            imm_fence,
            imm_command_pool,
            imm_command_buffer,
            //editor_data,
            gradient_pl,
            gradient_p,
            compute_effects:          vec![compute_effect_gradient, sky_effect],
            current_compute_effect:   1,
            mesh_system,
            texture_system,
            white_image:              INVALID_TEXTURE_ID,
            black_image:              INVALID_TEXTURE_ID,
            grey_image:               INVALID_TEXTURE_ID,
            error_checkerboard_image: INVALID_TEXTURE_ID,
            opaque_material,
            default_material:         INVALID_MATERIAL_INSTACE_ID, //todo
            view_matrix:              Float4x4::identity(),
            perspective_matrix:       Float4x4::identity(),
            outgoing_commands:        RenderCommandBuffer::default(),
        };

        // Let's create some test images
        //

        let white_image = {
            let packed_color = Float4::one().pack_unorm_u32();
            let packed_ptr = (&packed_color as *const u32) as *const u8;

            let texture_ci = TextureCreateInfo {
                name:   String::from("White Texture"),
                format: TextureFormat::R8g8b8a8Unorm,
                flags:  TextureFlags::MipMapped,
                sampler: SamplerType::Nearest,
                width:  1,
                height: 1,
                depth:  1,
                pixels: packed_ptr,
            };

            result.texture_system.create_texture(texture_ci, &mut result.imm_command_buffer, result.imm_fence)
        };

        let grey_image = {
            let packed_color = Float4::new(0.66, 0.66, 0.66, 1.0).pack_unorm_u32();
            let packed_ptr = (&packed_color as *const u32) as *const u8;

            let texture_ci = TextureCreateInfo {
                name:   String::from("Grey Texture"),
                format: TextureFormat::R8g8b8a8Unorm,
                flags:  TextureFlags::MipMapped,
                sampler: SamplerType::Nearest,
                width:  1,
                height: 1,
                depth:  1,
                pixels: packed_ptr,
            };

            result.texture_system.create_texture(texture_ci, &mut result.imm_command_buffer, result.imm_fence)
        };

        let black_image = {
            let packed_color = Float4::zero().pack_unorm_u32();
            let packed_ptr = (&packed_color as *const u32) as *const u8;

            let texture_ci = TextureCreateInfo {
                name:   String::from("Black Texture"),
                format: TextureFormat::R8g8b8a8Unorm,
                flags:  TextureFlags::MipMapped,
                sampler: SamplerType::Nearest,
                width:  1,
                height: 1,
                depth:  1,
                pixels: packed_ptr,
            };

            result.texture_system.create_texture(texture_ci, &mut result.imm_command_buffer, result.imm_fence)
        };

        let checkerboard = {
            let packed_magenta = Float4::new(1.0, 0.0, 1.0, 1.0).pack_unorm_u32();
            let packed_black   = Float4::zero().pack_unorm_u32();

            let mut pixels: [u32; 16*16] = [0; 16*16];
            for x in 0..16 {
          		for y in 0..16 {
         			pixels[y*16 + x] = if ((x % 2) ^ (y % 2) > 0) { packed_magenta } else { packed_black };
          		}
            }

            let texture_ci = TextureCreateInfo {
                name:   String::from("Error Checkerboard Texture"),
                format: TextureFormat::R8g8b8a8Unorm,
                flags:  TextureFlags::MipMapped,
                sampler: SamplerType::Nearest,
                width:  16,
                height: 16,
                depth:  1,
                pixels: pixels.as_ptr() as *const u8,
            };

            result.texture_system.create_texture(texture_ci, &mut result.imm_command_buffer, result.imm_fence)
        };

        result.white_image              = white_image;
        result.grey_image               = grey_image;
        result.black_image              = black_image;
        result.error_checkerboard_image = checkerboard;

        return result;
    }

    fn get_frame_data(&self) -> Rc<PerFrameData> {
        self.frame_data[self.swapchain.frame_index].clone()
    }

    fn draw_geometry(&mut self, cmd_buffer: &mut CommandBuffer, frame_state: &PerFrameState) {
        let scene_set = {
            let frame_data = self.get_frame_data();
            let mut dyn_descriptors = frame_data.dynamic_descriptors.borrow_mut();

            //create a descriptor set that binds that buffer and update it
           	let global_ds = dyn_descriptors.allocate(&self.device, self.opaque_material.scene_dl);

           	let mut writer = DescriptorWriter::new();
           	writer.write_buffer(0, frame_state.scene_data.gpu_buffer.buffer, std::mem::size_of::<GlobalSceneData>() as u64, 0, VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER);
           	writer.update_set(&self.device, global_ds);

            global_ds
        };

        // let's bind a texture!
        let image_set = {
            let texture_data = self.texture_system.get_texture_data(self.error_checkerboard_image)
                .expect("Failed to get error texture");

            let frame_data = self.get_frame_data();
            let mut dyn_descriptors = frame_data.dynamic_descriptors.borrow_mut();

           	let image_set = dyn_descriptors.allocate(&self.device, self.opaque_material.texture_resource_dl);

            let mut writer = DescriptorWriter::new();
           	writer.write_combined_image_sampler(0, texture_data.image.view, texture_data.sampler, VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL);
           	writer.update_set(&self.device, image_set);

            image_set
        };

        let color_attachment = make_color_attachment_info(self.scene_image.view, None, VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL);
       	let depth_attachment = make_depth_attachment_info(self.depth_image.view, VK_IMAGE_LAYOUT_DEPTH_ATTACHMENT_OPTIMAL);

        let draw_extent = VkExtent2D{ width: self.scene_image.dims.width, height: self.scene_image.dims.height };
        let render_info = make_rendering_info(draw_extent, &color_attachment, &depth_attachment);

        cmd_buffer.begin_rendering(render_info);

        cmd_buffer.bind_graphics_pipeline(self.opaque_material.pipeline);
        cmd_buffer.set_viewport(draw_extent.width as i32, (draw_extent.height  as i32), 0, 0);
        cmd_buffer.set_scissor(draw_extent.width, draw_extent.height);

        let sets: [VkDescriptorSet; 2] = [scene_set, image_set];
        cmd_buffer.bind_graphics_descriptor_sets(self.opaque_material.pipeline_layout, 0, &sets);

        let persp_view = mul_rh(self.perspective_matrix, self.view_matrix);

        let meshes_to_draw = self.mesh_system.collect_live_meshes();
        let draw_list = self.mesh_system.build_draw_list(frame_state.frame_index, &meshes_to_draw);

        for draw in draw_list {
            cmd_buffer.bind_push_constants(self.opaque_material.pipeline_layout, VK_SHADER_STAGE_VERTEX_BIT, draw.push_constant, 0);
            cmd_buffer.bind_index_buffer(&draw.index_buffer);
            cmd_buffer.draw_indexed(draw.index_count, 1, 0, 0, 0);
        }

        cmd_buffer.end_rendering();
    }

    fn process_render_commands(&mut self, command_buffer: &RenderCommandBuffer) {
        for command in &command_buffer.commands {
            match command {
                RenderCommand::UpdateCamera(camera) => {
                    self.view_matrix        = camera.view_matrix;
                    self.perspective_matrix = camera.perspective_matrix;
                }

                RenderCommand::CreateMesh(mesh_info) => {
                    let mesh_ci = MeshCreateInfo {
                        vertices:     mesh_info.vertices,
                        vertex_count: mesh_info.vertex_count,
                        indices:      mesh_info.indices,
                        index_count:  mesh_info.index_count,
                        transform:    mesh_info.transform,
                        material:     self.default_material,
                    };

                    let mesh_id = self.mesh_system.create_mesh(mesh_ci, &mut self.imm_command_buffer, self.imm_fence);

                    let response = ReadyMeshInfo{
                        engine_id:      mesh_info.engine_id,
                        render_mesh_id: RenderId::Mesh(mesh_id),
                    };

                    self.outgoing_commands.commands.push_back(RenderCommand::ReadyMesh(response));
                },

                default => {},
            }
        }
    }

    pub fn submit_render_commands(&mut self, render_command_buffer: RenderCommandBuffer) {
        self.process_render_commands(&render_command_buffer);
    }

    pub fn render(&mut self) {
        // If the swapchain has been invalidated, recreate it. Will usually happen when we need to resize.
        if !self.swapchain.is_valid()
        {
            self.resize_device_resources();
            return; //don't render this frame...
        }

        if !self.swapchain.acquire_frame(&self.device) {
            return; // try again next frame
        }

        let frame_data = self.get_frame_data();

        // Process per-frame garbage
        //

        {
            let mut dyn_descriptors = frame_data.dynamic_descriptors.borrow_mut();
            dyn_descriptors.clear_pools(&self.device);
        }

        {
            let mut deletion_queues = frame_data.deletion_queues.borrow_mut();
            for buffer in &mut deletion_queues.buffer_deletion_queue {
                self.device.destroy_buffer(buffer);
            }

            for image in &mut deletion_queues.image_deletion_queue {
                self.device.destroy_image_memory(image);
            }

            deletion_queues.buffer_deletion_queue.clear();
            deletion_queues.image_deletion_queue.clear();
        }

        // Update Read-Only state
        //

        {
            //todo: this should be done with the update camera info
            let mut state = frame_data.state.borrow_mut();
            state.frame_index = self.swapchain.get_swapchain_frame_index();
            state.scene_data.scene.view      = self.view_matrix;
            state.scene_data.scene.proj      = self.perspective_matrix;
            state.scene_data.scene.view_proj = mul_rh(self.perspective_matrix, self.view_matrix);

            // copy the scene data into the uniform buffer
            let buffer_ptr = state.scene_data.gpu_buffer.get_allocation() as *mut GlobalSceneData;
            unsafe { *buffer_ptr = state.scene_data.scene };
        }

        // Render the Frame
        //

        let mut command_buffer_state = frame_data.command_buffer.borrow_mut();
        let mut command_buffer = &mut command_buffer_state.handle;

        command_buffer.reset();
        command_buffer.begin_recording();

        command_buffer.transition_image(self.scene_image.image, VK_IMAGE_LAYOUT_UNDEFINED, VK_IMAGE_LAYOUT_GENERAL);

        {
            let compute_effect = &self.compute_effects[self.current_compute_effect];

            command_buffer.bind_compute_pipeline(compute_effect.pipeline);

            let descriptors: [VkDescriptorSet; 1] = [ self.draw_image_ds ];
            command_buffer.bind_compute_descriptor_sets(compute_effect.layout, 0, descriptors.as_slice());

            command_buffer.bind_push_constants(compute_effect.layout, VK_SHADER_STAGE_COMPUTE_BIT, compute_effect.push_data, 0);

            let group_x = self.scene_image.dims.width  as f32 / 16.0;
            let group_y = self.scene_image.dims.height as f32 / 16.0;

            command_buffer.dispatch_compute(group_x.ceil() as u32, group_y.ceil() as u32, 1);
        }

        { // Draw geometry
            command_buffer.transition_image(self.scene_image.image, VK_IMAGE_LAYOUT_GENERAL,   VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL);
            command_buffer.transition_image(self.depth_image.image, VK_IMAGE_LAYOUT_UNDEFINED, VK_IMAGE_LAYOUT_DEPTH_ATTACHMENT_OPTIMAL);

            self.draw_geometry(&mut command_buffer, &frame_data.state.borrow());
        }

        // Now, copy the scene framebuffer to the swapchain
        let swapchain_image      = self.swapchain.get_swapchain_image();
        let swapchain_image_view = self.swapchain.get_swapchain_image_view();
        let swapchain_extent     = self.swapchain.get_extent();

        command_buffer.transition_image(self.scene_image.image, VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL, VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL);
        command_buffer.transition_image(swapchain_image,        VK_IMAGE_LAYOUT_UNDEFINED,                VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL);

        let src_extent = VkExtent2D{ width: self.scene_image.dims.width, height: self.scene_image.dims.height };
        let dst_extent = VkExtent2D{ width: swapchain_extent.width,      height: swapchain_extent.height      };

        command_buffer.copy_image_to_image(self.scene_image.image, src_extent, swapchain_image, dst_extent);

        // Render Imgui directly into the swapchain image.
        //   note: it is likely I will want to render into a rgba 8bit target and composite with the scene image
        //         before copying into the swapchain buffer. vkguide does this, so I am going to do this for now.
        //command_buffer.transition_image(swapchain_image, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL);

        // Transition the swapchain image to present mode
        command_buffer.transition_image(swapchain_image, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, VK_IMAGE_LAYOUT_PRESENT_SRC_KHR);

        // End the Frame
        //

        command_buffer.end_recording();

        let cmd_buffer_si = command_buffer.get_submit_info();

        let render_sem  = self.swapchain.get_render_semaphore();
        let present_sem = self.swapchain.get_present_semaphore();

        // Want to wait on the PresentSemaphore, as that semaphore is signaled when the swapchain is ready
        let wait_info   = make_semaphore_submit_info(VK_PIPELINE_STAGE_2_COLOR_ATTACHMENT_OUTPUT_BIT_KHR, present_sem);
        // Will signal the renderSemaphore, to signal that rendering has finished
        let signal_info = make_semaphore_submit_info(VK_PIPELINE_STAGE_2_ALL_GRAPHICS_BIT, render_sem);

        let submit = make_submit_info(cmd_buffer_si, Some(signal_info), Some(wait_info));

        // Submit command buffer to the queue and execute it.
        //   renderFence will now block until the graphic commands finish execution
        let graphics_queue = self.device.get_queue(QueueType::Graphics);

        call_throw!(self.device.fns.queue_submit2, graphics_queue, 1, &submit, self.swapchain.get_render_fence());

        // todo: grab an "empty" command buffer to wait on currentFrameData->mPresentSemaphore
        // vkAcquireImageKHR will signal this semaphore when we are ready to render into this image.
        // In a real graphics pipeline, we might want to do this in the compositing step when we render
        // directly into the swapchain Framebuffer.

        self.swapchain.present_frame(&self.device);

        self.frame_index = (self.frame_index + 1) % consts::MAX_BUFFERED_FRAMES;
    }

    pub fn destroy(&mut self) {
        self.device.wait_idle();

        self.mesh_system.destroy();

        self.texture_system.destroy_texture(self.white_image);
        self.texture_system.destroy_texture(self.black_image);
        self.texture_system.destroy_texture(self.grey_image);
        self.texture_system.destroy_texture(self.error_checkerboard_image);
        self.texture_system.destroy();

        self.opaque_material.on_destroy(&self.device);

        self.device.destroy_pipeline(self.gradient_p);
        self.device.destroy_pipeline_layout(self.gradient_pl);

        for effect in &self.compute_effects {
            self.device.destroy_pipeline(effect.pipeline);
            self.device.destroy_pipeline_layout(effect.layout);
        }

        self.device.destroy_descriptor_allocator(&mut self.global_da);
        self.device.destroy_descriptor_set_layout(self.draw_image_dl);
        self.device.destroy_descriptor_set_layout(self.global_scene_dl);

        self.device.destroy_fence(&mut self.imm_fence);
        self.device.destroy_command_pool(&mut self.imm_command_pool);

        for frame_data in &self.frame_data{
            self.device.destroy_command_pool(&mut frame_data.command_buffer.borrow_mut().pool);

            {
                let mut dyn_descriptors = frame_data.dynamic_descriptors.borrow_mut();
                dyn_descriptors.destroy(&self.device);
            }

            {
                let mut deletion_queues = frame_data.deletion_queues.borrow_mut();
                for buffer in &mut deletion_queues.buffer_deletion_queue {
                    self.device.destroy_buffer(buffer);
                }

                for image in &mut deletion_queues.image_deletion_queue {
                    self.device.destroy_image_memory(image);
                }

                deletion_queues.buffer_deletion_queue.clear();
                deletion_queues.image_deletion_queue.clear();
            }

            {
                let mut scene = frame_data.state.borrow_mut();
                self.device.destroy_buffer(&mut scene.scene_data.gpu_buffer);
            }
        }

        self.device.destroy_image_memory(&mut self.depth_image);
        self.device.destroy_image_memory(&mut self.scene_image);
        self.device.destroy_swapchain(&mut self.swapchain);
        //device will be dropped last
    }

    pub fn on_resize(&mut self, width: u32, height: u32)
    {
        self.swapchain.on_resize(width, height);
    }
}

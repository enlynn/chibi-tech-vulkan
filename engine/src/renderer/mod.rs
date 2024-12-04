mod graphics;

use crate::math::{
    float3::*,
    float4::*,
    float4x4::*,
};
use crate::util::ffi::*;

use gpu_descriptors::DescriptorAllocatorFlags;
use graphics::*;
use graphics::{
    AllocatedImage,
    gpu_device::Device,
    gpu_swapchain::Swapchain,
    gpu_utils::*,
    gpu_command_pool::CommandPool,
    gpu_command_buffer::CommandBuffer,
    gpu_descriptors::{DescriptorAllocator, DescriptorLayoutBuilder, PoolSizeRatio},
    gpu_pipeline::*,
};

use super::window::NativeSurface;

use std::borrow::BorrowMut;
use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;
use std::str::FromStr;
use std::collections::VecDeque;

use vendor::vulkan::*;
use vendor::imgui::*;

// Render Commands
//

//#[derive(Clone, Copy)]
pub struct CreateMeshInfo {
    pub vertices:     *const Vertex,
    pub vertex_count: usize,

    pub indices:      *const u32,
    pub index_count:  usize,

    //todo: other mesh properties

    // Some engine-id so that we can tell the engine the mesh has uploaded
    pub engine_id:    u64,
}

pub struct ReadyMeshInfo {
    engine_id:      u64,
    render_mesh_id: u64,
}

pub enum RenderCommand{
    // Engine -> Renderer Commands
    //

    // Mesh-related commands
    CreateMesh(CreateMeshInfo),
    DestroyMesh,
    HideMesh,
    ShowMesh,

    // Texture-related commands
    CreateTexture,
    DestroyTexture,

    // Material-related commands
    CreateMaterial,
    DestroyMaterial,

    // Renderer -> Engine Commands
    //

    ReadyMesh(ReadyMeshInfo),
}

pub struct RenderCommandBuffer{
    commands: VecDeque<RenderCommand>,
}

impl Default for RenderCommandBuffer{
    fn default() -> Self{
        Self{
            commands: VecDeque::<RenderCommand>::new(),
        }
    }
}

impl RenderCommandBuffer {
    pub fn add_command(&mut self, cmd: RenderCommand) {
        self.commands.push_back(cmd);
    }
}

// Per Frame State
//

struct PerFrameState{
    command_pool:   CommandPool,
    command_buffer: CommandBuffer,
}

struct PerFrameData {
    state: RefCell<PerFrameState>, // i don't like this one bit...
}

// Compute Effects
//

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct ComputePushConstants {
    data1: Float4,
    data2: Float4,
    data3: Float4,
    data4: Float4,
}

struct ComputeEffect {
    pub name:      String,
	pub pipeline:  VkPipeline,
	pub layout:    VkPipelineLayout,
	pub push_data: ComputePushConstants,
}

// Mesh info
//

pub type MeshId = usize;

#[repr(C)]
#[derive(Clone, Copy)]
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

#[derive(Clone, Copy)]
struct GpuMeshBuffers {
    index_buffer:          AllocatedBuffer,
    vertex_buffer:         AllocatedBuffer,
    vertex_buffer_address: VkDeviceAddress,
    index_count:           u32,
}

struct GpuDrawPushConstants {
    world_matrix:  Float4x4,
    vertex_buffer: VkDeviceAddress,
}

// Render System
//

const MAX_LOADED_MESHES: usize = 100;

pub struct RendererCreateInfo {
    pub surface: NativeSurface,
}

pub struct RenderSystem{
    device:      Device,
    swapchain:   Swapchain,
    scene_image: AllocatedImage,
    depth_image: AllocatedImage,

    frame_data:  Vec<Rc<PerFrameData>>,
    frame_index: usize,

    global_da:     DescriptorAllocator,
    draw_image_dl: VkDescriptorSetLayout,
    draw_image_ds: VkDescriptorSet,

    // immediate context submission - not quite sure where to put this right now
    //   This is largely used for an Upload Context for pushing data to the GPU.
    //   Ideally, I would be using some sort of paged heap to push data instead
    //   of waiting for every single mesh to upload before moving on.
    imm_fence:          Fence,
    imm_command_buffer: CommandBuffer,
    imm_command_pool:   CommandPool,

    // IMGUI Editor Data
    editor_data:        EditorRenderData,

	// for the background
	gradient_pl:   VkPipelineLayout,
	gradient_p:    VkPipeline,

	compute_effects:        Vec<ComputeEffect>,
	current_compute_effect: usize,

	// for the triangle
	triangle_pl:   VkPipelineLayout,
	triangle_p:    VkPipeline,

	// Mesh "System"
	meshes:        [GpuMeshBuffers; MAX_LOADED_MESHES],
	mesh_count:    usize,

	// Outgoing Commands to the engine
	//   Will probably want this as a mpsc::Sender once the Renderer gets put on its own thread.
	outgoing_commands: RenderCommandBuffer,
}

enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

impl Default for GpuMeshBuffers {
    fn default() -> Self {
        Self{
            index_buffer:          AllocatedBuffer::default(),
            vertex_buffer:         AllocatedBuffer::default(),
            vertex_buffer_address: 0,
            index_count:           0,
        }
    }
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

fn load_shader_module(device: &Device, shader_name: &str, stage: ShaderStage) -> VkShaderModule {
    use crate::core::asset_system::{AssetDrive, AssetSystem};
    use std::io::prelude::*;
    use std::fs::File;

    let asset_dir  = AssetSystem::get_root_dir(AssetDrive::Priv);

    //todo: cache this so we don't have to recreate it for every shader
    let shader_dir  = asset_dir.join("shaders/.cache");

    let mut shader_name_str = String::from_str(shader_name).expect("Failed to construct string.");
    match stage {
        ShaderStage::Vertex   => { shader_name_str.push_str(".vert.spv"); },
        ShaderStage::Fragment => { shader_name_str.push_str(".frag.spv"); },
        ShaderStage::Compute  => { shader_name_str.push_str(".comp.spv"); },
    };

    let shader_file = shader_dir.join(shader_name_str);
    let display = shader_file.display();

    println!("Shader Cache Directory: {:?}", shader_file);

    // let's read the file
    let mut file = match File::open(&shader_file) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut file_data = Vec::<u8>::new();
    match file.read_to_end(&mut file_data) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_)    => {},
    }

    device.create_shader_module(file_data.as_slice()).expect("Failed to create VkShaderModule from gradient.spv")
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
            VK_IMAGE_ASPECT_COLOR_BIT)
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
            VK_IMAGE_ASPECT_DEPTH_BIT,
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
            let ds = self.device.allocate_descriptors(&self.global_da, self.draw_image_dl);

            let mut image_info = VkDescriptorImageInfo::default();
            image_info.imageLayout = VK_IMAGE_LAYOUT_GENERAL;
            image_info.imageView   = self.scene_image.view;

            self.device.update_descriptor_sets(image_info, ds, 1, 0, 0, VK_DESCRIPTOR_TYPE_STORAGE_IMAGE);

            ds
        };

        vendor::imgui::ig_vulkan_set_min_image_count(self.swapchain.get_image_count() as u32);
        //ImGui_ImplVulkanH_CreateOrResizeWindow(g_Instance, g_PhysicalDevice, g_Device, &g_MainWindowData, g_QueueFamily, g_Allocator, fb_width, fb_height, g_MinImageCount);
    }

    pub fn new(create_info: RendererCreateInfo) -> RenderSystem {
        let device = Device::new(gpu_device::CreateInfo{
            features:         gpu_device::Features::default(),  //todo: make configurable
            surface:          create_info.surface,
            software_version: crate::make_app_version(0, 0, 1), //todo: make configurable
            software_name:    String::from("Testbed"),          //todo: make configurable
        });

        let swapchain = device.create_swapchain(None);

        let scene_image = RenderSystem::create_scene_images(&device, swapchain.get_extent());
        let depth_image = RenderSystem::create_depth_image(&device,  swapchain.get_extent());

        let init_frame_data = |device: &Device| -> PerFrameData {
            let command_pool =   device.create_command_pool(QueueType::Graphics);
            let command_buffer = device.create_command_buffer(&command_pool);
            PerFrameData{ state: RefCell::new(PerFrameState { command_pool, command_buffer }) }
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
            let ds = device.allocate_descriptors(&global_da, draw_image_dl);

            let mut image_info = VkDescriptorImageInfo::default();
            image_info.imageLayout = VK_IMAGE_LAYOUT_GENERAL;
            image_info.imageView   = scene_image.view;

            device.update_descriptor_sets(image_info, ds, 1, 0, 0, VK_DESCRIPTOR_TYPE_STORAGE_IMAGE);

            ds
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

        let colored_tri_vert_sm = load_shader_module(&device, "colored_triangle", ShaderStage::Vertex);
        let colored_tri_frag_sm = load_shader_module(&device, "colored_triangle", ShaderStage::Fragment);

        let triangle_pl = {
            let descriptors:    [VkDescriptorSetLayout; 0] = [];
            let push_constants: [VkPushConstantRange;   1] = [
                VkPushConstantRange{
                    stageFlags: VK_SHADER_STAGE_VERTEX_BIT,
                    offset:     0,
                    size:       std::mem::size_of::<GpuDrawPushConstants>() as u32,
                },
            ];

            device.create_pipeline_layout(descriptors.as_slice(), push_constants.as_slice())
        };

        let triangle_p = {
            let mut builder = GraphicsPipelineBuilder::new();

            //use the triangle layout we created
            builder
                .set_pipeline_layout(triangle_pl)
            //connecting the vertex and pixel shaders to the pipeline
                .set_shaders(colored_tri_vert_sm, colored_tri_frag_sm)
            //it will draw triangles
                .set_input_topology(VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST)
            //filled triangles
                .set_polygon_mode(VK_POLYGON_MODE_FILL)
            //no backface culling
                .set_cull_mode(VK_CULL_MODE_NONE, VK_FRONT_FACE_CLOCKWISE)
            //no multisampling
                .set_multisampling_none()
            //no blending
                //.disable_blending()
            // additive blending
                .enabled_blending_additive()
            // alpha blending
                //.enabled_blending_alphablend()
            //no depth testing
                //.disable_depth_test()
            // enabled depth testing
                .enable_depth_test(true, VK_COMPARE_OP_GREATER_OR_EQUAL)
            //connect the image format we will draw into, from draw image
                .set_color_attachment_format(scene_image.format)
                .set_depth_format(depth_image.format);

            //finally build the pipeline
            builder.build(&device)
        };

        device.destroy_shader_module(colored_tri_vert_sm);
        device.destroy_shader_module(colored_tri_frag_sm);

        // Setup imgui
        //
        let editor_data = device.create_imgui_editor(swapchain.get_image_count() as u32);

        return RenderSystem{
            device,
            swapchain,
            scene_image,
            depth_image,
            frame_data,
            frame_index: 0,
            global_da,
            draw_image_dl,
            draw_image_ds,
            imm_fence,
            imm_command_pool,
            imm_command_buffer,
            editor_data,
            gradient_pl,
            gradient_p,
            compute_effects:        vec![compute_effect_gradient, sky_effect],
            current_compute_effect: 1,
            triangle_pl,
            triangle_p,
            meshes:            [GpuMeshBuffers::default(); MAX_LOADED_MESHES],
            mesh_count:        0,
            outgoing_commands: RenderCommandBuffer::default(),
        };
    }

    fn get_frame_data(&self) -> Rc<PerFrameData> {
        self.frame_data[self.swapchain.frame_index].clone()
    }

    fn draw_geometry(&self, cmd_buffer: &mut CommandBuffer) {
        let color_attachment = make_color_attachment_info(self.scene_image.view, None, VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL);
       	let depth_attachment = make_depth_attachment_info(self.depth_image.view, VK_IMAGE_LAYOUT_DEPTH_ATTACHMENT_OPTIMAL);

        let draw_extent = VkExtent2D{ width: self.scene_image.dims.width, height: self.scene_image.dims.height };
        let render_info = make_rendering_info(draw_extent, &color_attachment, &depth_attachment);

        cmd_buffer.begin_rendering(render_info);

        cmd_buffer.bind_graphics_pipeline(self.triangle_p);
        cmd_buffer.set_viewport(draw_extent.width, draw_extent.height, 0, 0);
        cmd_buffer.set_scissor(draw_extent.width, draw_extent.height);
        //cmd_buffer.draw(3, 1, 0, 0);

        for i in 0..self.mesh_count {
            let mesh = &self.meshes[i];

            let push_consts = GpuDrawPushConstants {
                world_matrix: Float4x4::identity(),
                vertex_buffer: mesh.vertex_buffer_address,
            };

            cmd_buffer.bind_push_constants(self.triangle_pl, VK_SHADER_STAGE_VERTEX_BIT, push_consts, 0);
            cmd_buffer.bind_index_buffer(&mesh.index_buffer);
            cmd_buffer.draw_indexed(mesh.index_count, 1, 0, 0, 0);
        }

        cmd_buffer.end_rendering();
    }

    // A function which takes the closure: fn func(cmd_buffer: &CommandBuffer)
    fn immediate_submit<F>(&mut self, f: F) where
        F: Fn(&CommandBuffer)
    {
        self.device.reset_fences(&self.imm_fence);
        self.imm_command_buffer.reset();
        self.imm_command_buffer.begin_recording();

        // execute the function
        f(&self.imm_command_buffer);

        self.imm_command_buffer.end_recording();

        let cmd_buffer_si = self.imm_command_buffer.get_submit_info();
        let submit = make_submit_info(cmd_buffer_si, None, None);

        self.device.queue_submit(QueueType::Graphics, submit, self.imm_fence);
        self.device.wait_for_fences(self.imm_fence);
    }

    pub fn render_editor(&mut self, command_buffer: &mut CommandBuffer, image_view: VkImageView) {
        use crate::util::ffi::*;

        call!(vendor::imgui::igRender);

        let draw_extent = VkExtent2D{ width: self.scene_image.dims.width, height: self.scene_image.dims.height };
        let color_attachment = make_color_attachment_info(image_view, None, VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL);
        let render_info = make_rendering_info(draw_extent, &color_attachment, std::ptr::null());

        command_buffer.begin_rendering(render_info);
        vendor::imgui::ig_vulkan_render_draw_data(command_buffer.handle, ptr::null_mut());
        command_buffer.end_rendering();
    }

    pub fn on_editor_update(&mut self) {
        use std::{ffi::CString, os::raw};

        let window_name = CString::new("Background").expect("Failed to convert name to CString.");
        let mut window_open = true;

        if call!(igBegin, window_name.as_ptr(), &mut window_open, 0) {
            let max_effects = (self.compute_effects.len() - 1) as i32;

            let compute_effect = &mut self.compute_effects[self.current_compute_effect];

            let select_name     = CString::new(format!("Selected Name: {}", compute_effect.name)).expect("Failed to convert name to CString.");
            let effect_idx_name = CString::new("Effect Index").expect("Failed to convert name to CString.");
            let data1_name      = CString::new("Data 1").expect("Failed to convert name to CString.");
            let data2_name      = CString::new("Data 2").expect("Failed to convert name to CString.");
            let data3_name      = CString::new("Data 3").expect("Failed to convert name to CString.");
            let data4_name      = CString::new("Data 4").expect("Failed to convert name to CString.");

			call!(igText, select_name.as_ptr());
			call!(igSliderInt, effect_idx_name.as_ptr(), &mut self.current_compute_effect as *mut usize as *mut i32,
			    0, max_effects as i32, ptr::null(), 0);

			call!(igDragFloat4, data1_name.as_ptr(), &mut compute_effect.push_data.data1.x, 0.1, 0.0, 1.0, ptr::null(), 0);
			call!(igDragFloat4, data2_name.as_ptr(), &mut compute_effect.push_data.data2.x, 0.1, 0.0, 1.0, ptr::null(), 0);
			call!(igDragFloat4, data3_name.as_ptr(), &mut compute_effect.push_data.data3.x, 0.1, 0.0, 1.0, ptr::null(), 0);
			call!(igDragFloat4, data4_name.as_ptr(), &mut compute_effect.push_data.data4.x, 0.1, 0.0, 1.0, ptr::null(), 0);
        }

        call!(igEnd);
    }

    fn process_render_commands(&mut self, command_buffer: &RenderCommandBuffer) {
        for command in &command_buffer.commands {
            match command {
                RenderCommand::CreateMesh(mesh_info) => {
                    assert!(self.mesh_count < MAX_LOADED_MESHES - 1);

                    let vertices = unsafe { std::slice::from_raw_parts(mesh_info.vertices, mesh_info.vertex_count) };
                    let indices  = unsafe { std::slice::from_raw_parts(mesh_info.indices,  mesh_info.index_count)  };

                    //note: this will evventually be deferred.
                    let mesh = self.upload_mesh(indices, vertices);

                    let mesh_id = self.mesh_count;

                    self.meshes[self.mesh_count] = mesh;
                    self.mesh_count += 1;

                    let response = ReadyMeshInfo{
                        engine_id:      mesh_info.engine_id,
                        render_mesh_id: mesh_id as u64,
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

    pub fn render(&mut self, render_command_buffer: RenderCommandBuffer) {
        if render_command_buffer.commands.len() > 0 {
            self.process_render_commands(&render_command_buffer);
        }

        // If the swapchain has been invalidated, recreate it. Will usually happen when we need to resize.
        if !self.swapchain.is_valid()
        {
            self.resize_device_resources();
            return; //don't render this frame...
        }

        if !self.swapchain.acquire_frame(&self.device) {
            return; // try again next frame
        }

        // todo: process per-frame garbage
        //

        // Render the Frame
        //

        let frame_data  = self.get_frame_data();
        let mut frame_state = frame_data.state.borrow_mut();

        let mut command_buffer = &mut frame_state.command_buffer;

        command_buffer.reset();
        command_buffer.begin_recording();

        command_buffer.transition_image(self.scene_image.image, VK_IMAGE_LAYOUT_UNDEFINED, VK_IMAGE_LAYOUT_GENERAL);

        if false { // Draw background, simple
            //command_buffer.clear_color_image(self.scene_image.image, &clear_value);

            command_buffer.bind_compute_pipeline(self.gradient_p);

            let descriptors: [VkDescriptorSet; 1] = [ self.draw_image_ds ];
            command_buffer.bind_compute_descriptor_sets(self.gradient_pl, 0, descriptors.as_slice());

            let group_x = self.scene_image.dims.width  as f32 / 16.0;
            let group_y = self.scene_image.dims.height as f32 / 16.0;

            command_buffer.dispatch_compute(group_x.ceil() as u32, group_y.ceil() as u32, 1);
        } else {
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

            self.draw_geometry(&mut command_buffer);
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
        command_buffer.transition_image(swapchain_image, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL);
        self.render_editor(&mut command_buffer, swapchain_image_view);

        // Transition the swapchain image to present mode
        command_buffer.transition_image(swapchain_image, VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL, VK_IMAGE_LAYOUT_PRESENT_SRC_KHR);

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

        for i in 0..self.mesh_count {
            let mesh = &mut self.meshes[i];
            self.device.destroy_buffer(&mut mesh.index_buffer);
            self.device.destroy_buffer(&mut mesh.vertex_buffer);
        }

        self.device.destroy_imgui_editor(&mut self.editor_data);

        self.device.destroy_pipeline(self.triangle_p);
        self.device.destroy_pipeline_layout(self.triangle_pl);

        self.device.destroy_pipeline(self.gradient_p);
        self.device.destroy_pipeline_layout(self.gradient_pl);

        for effect in &self.compute_effects {
            self.device.destroy_pipeline(effect.pipeline);
            self.device.destroy_pipeline_layout(effect.layout);
        }

        self.device.destroy_descriptor_allocator(&mut self.global_da);
        self.device.destroy_descriptor_set_layout(self.draw_image_dl);

        self.device.destroy_fence(&mut self.imm_fence);
        self.device.destroy_command_pool(&mut self.imm_command_pool);

        for frame_data in &self.frame_data{
            self.device.destroy_command_pool(&mut frame_data.state.borrow_mut().command_pool);
        }

        self.device.destroy_image_memory(&mut self.depth_image);
        self.device.destroy_image_memory(&mut self.scene_image);
        self.device.destroy_swapchain(&mut self.swapchain);
        self.device.destroy();
    }

    pub fn on_resize(&mut self, width: u32, height: u32)
    {
        self.swapchain.on_resize(width, height);
    }

    fn upload_mesh(&mut self, indices: &[u32], vertices: &[Vertex]) -> GpuMeshBuffers {
        let vertex_buffer_size = vertices.len() * std::mem::size_of::<Vertex>();
        let index_buffer_size  = indices.len()  * std::mem::size_of::<u32>();

        let vertex_buffer_flags = VK_BUFFER_USAGE_STORAGE_BUFFER_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT | VK_BUFFER_USAGE_SHADER_DEVICE_ADDRESS_BIT;
        let index_buffer_flags  = VK_BUFFER_USAGE_INDEX_BUFFER_BIT | VK_BUFFER_USAGE_TRANSFER_DST_BIT;

        let mut result = GpuMeshBuffers::default();
        result.index_buffer          = self.device.create_buffer(index_buffer_size, index_buffer_flags, VMA_MEMORY_USAGE_GPU_ONLY);
        result.vertex_buffer         = self.device.create_buffer(vertex_buffer_size, vertex_buffer_flags, VMA_MEMORY_USAGE_GPU_ONLY);
        result.vertex_buffer_address = self.device.get_buffer_device_address(&result.vertex_buffer);
        result.index_count           = indices.len() as u32;

       	let mut staging_buffer = self.device.create_buffer(vertex_buffer_size + index_buffer_size, VK_BUFFER_USAGE_TRANSFER_SRC_BIT, VMA_MEMORY_USAGE_CPU_ONLY);

        let mut memory = staging_buffer.info.pMappedData;
        assert!(memory != ptr::null_mut());

        // Copy vertex data to the staging buffer
        let mut memory_as_vertex = memory as *mut Vertex;
        unsafe { std::ptr::copy(vertices.as_ptr(), memory_as_vertex, vertices.len()) };

        // Copy index data to the staging buffer
        let mut memory_as_index = unsafe { memory.add(vertex_buffer_size) } as *mut u32;
        unsafe { std::ptr::copy(indices.as_ptr(), memory_as_index, indices.len()) };

        self.immediate_submit(
            |command_buffer: &CommandBuffer| {
                // Copy to the final vertex buffer
                command_buffer.copy_buffer(&result.vertex_buffer, 0, &staging_buffer, 0, vertex_buffer_size as VkDeviceSize);
                // Copy to the final index buffer
                command_buffer.copy_buffer(&result.index_buffer, 0, &staging_buffer, vertex_buffer_size as VkDeviceSize, index_buffer_size as VkDeviceSize);
            }
        );

        self.device.destroy_buffer(&mut staging_buffer);

        return result;
    }
}

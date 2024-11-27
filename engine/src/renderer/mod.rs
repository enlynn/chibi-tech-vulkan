mod graphics;

use api::{VkDescriptorSetLayout, VkExtent3D};
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
use std::rc::Rc;
use std::cell::RefCell;
use std::str::FromStr;

#[derive(Clone, Copy)]
pub enum RenderCommand{
    // Mesh-related commands
    CreateMesh,
    DestroyMesh,
    HideMesh,
    ShowMesh,

    // Texture-related commands
    CreateTexture,
    DestroyTexture,

    // Material-related commands
    CreateMaterial,
    DestroyMaterial,
}

pub struct RenderCommandBuffer{
    commands: Vec<RenderCommand>,
}

impl Default for RenderCommandBuffer{
    fn default() -> Self{
        Self{
            commands: Vec::<RenderCommand>::new(),
        }
    }
}

pub struct RendererCreateInfo {
    pub surface: NativeSurface,
}

struct PerFrameState{
    command_pool:   CommandPool,
    command_buffer: CommandBuffer,
}

struct PerFrameData {
    state: RefCell<PerFrameState>, // i don't like this one bit...
}

pub struct RenderSystem{
    device:      Device,
    swapchain:   Swapchain,
    scene_image: AllocatedImage,

    frame_data:  Vec<Rc<PerFrameData>>,
    frame_index: usize,

    silly: usize,

    global_da:     DescriptorAllocator,
    draw_image_dl: api::VkDescriptorSetLayout,
	draw_image_ds: api::VkDescriptorSet,

	// for the background
	gradient_pl:   api::VkPipelineLayout,
	gradient_p:    api::VkPipeline,

	// for the triangle
	triangle_pl:   api::VkPipelineLayout,
	triangle_p:    api::VkPipeline,
}

enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

fn load_shader_module(device: &Device, shader_name: &str, stage: ShaderStage) -> api::VkShaderModule {
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
        let image_usages: api::VkImageUsageFlags =
            api::VK_IMAGE_USAGE_TRANSFER_SRC_BIT |
            api::VK_IMAGE_USAGE_TRANSFER_DST_BIT |
            api::VK_IMAGE_USAGE_STORAGE_BIT      |
            api::VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT;

        device.allocate_image_memory(
            extent,
            api::VK_FORMAT_R16G16B16A16_SFLOAT,
            image_usages,
            api::VMA_MEMORY_USAGE_GPU_ONLY,
            api::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            api::VK_IMAGE_ASPECT_COLOR_BIT)
    }

    fn resize_device_resources(&mut self) {
        self.device.wait_idle();

        self.swapchain = self.device.create_swapchain(Some(&self.swapchain));
        self.swapchain.validate();

        self.device.destroy_image_memory(&mut self.scene_image);
        self.scene_image = RenderSystem::create_scene_images(&self.device, self.swapchain.get_extent());

        self.device.clear_descriptor_allocator(&self.global_da);
        self.draw_image_ds = {
            let ds = self.device.allocate_descriptors(&self.global_da, self.draw_image_dl);

            let mut image_info = api::VkDescriptorImageInfo::default();
            image_info.imageLayout = api::VK_IMAGE_LAYOUT_GENERAL;
            image_info.imageView   = self.scene_image.view;

            self.device.update_descriptor_sets(image_info, ds, 1, 0, 0, api::VK_DESCRIPTOR_TYPE_STORAGE_IMAGE);

            ds
        };
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

        let init_frame_data = |device: &Device| -> PerFrameData {
            let command_pool =   device.create_command_pool(QueueType::Graphics);
            let command_buffer = device.create_command_buffer(&command_pool);
            PerFrameData{ state: RefCell::new(PerFrameState { command_pool, command_buffer }) }
        };

        let mut frame_data = Vec::<Rc<PerFrameData>>::with_capacity(swapchain.images.len());
        for i in 0..swapchain.images.len() {
            frame_data.push(Rc::new(init_frame_data(&device)));
        }

        // Create descriptors
        //

        let global_da = {
            let sizes: [PoolSizeRatio; 1] = [
                PoolSizeRatio{descriptor_type: api::VK_DESCRIPTOR_TYPE_STORAGE_IMAGE, ratio: 1.0 },
            ];

            device.create_descriptor_allocator(10, sizes.as_slice())
        };

        let draw_image_dl = {
            let mut builder = DescriptorLayoutBuilder::new();
            builder.add_binding(0, api::VK_DESCRIPTOR_TYPE_STORAGE_IMAGE);

            builder.build(&device, api::VK_SHADER_STAGE_COMPUTE_BIT, 0)
        };

        let draw_image_ds = {
            let ds = device.allocate_descriptors(&global_da, draw_image_dl);

            let mut image_info = api::VkDescriptorImageInfo::default();
            image_info.imageLayout = api::VK_IMAGE_LAYOUT_GENERAL;
            image_info.imageView   = scene_image.view;

            device.update_descriptor_sets(image_info, ds, 1, 0, 0, api::VK_DESCRIPTOR_TYPE_STORAGE_IMAGE);

            ds
        };

        // The Compute Pipeline
        //

        let gradient_sm = load_shader_module(&device, "gradient", ShaderStage::Compute);

        let gradient_pl = {
            let descriptors:    [api::VkDescriptorSetLayout; 1] = [ draw_image_dl ];
            let push_constants: [api::VkPushConstantRange; 0]   = [];

            device.create_pipeline_layout(descriptors.as_slice(), push_constants.as_slice())
        };

        let gradient_p = device.create_compute_pipeline(gradient_sm, gradient_pl);

        device.destroy_shader_module(gradient_sm);

        // Colored Triangle Pipeline
        //

        let colored_tri_vert_sm = load_shader_module(&device, "colored_triangle", ShaderStage::Vertex);
        let colored_tri_frag_sm = load_shader_module(&device, "colored_triangle", ShaderStage::Fragment);

        let triangle_pl = {
            let descriptors:    [api::VkDescriptorSetLayout; 0] = [];
            let push_constants: [api::VkPushConstantRange;   0] = [];

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
                .set_input_topology(api::VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST)
            //filled triangles
                .set_polygon_mode(api::VK_POLYGON_MODE_FILL)
            //no backface culling
                .set_cull_mode(api::VK_CULL_MODE_NONE, api::VK_FRONT_FACE_CLOCKWISE)
            //no multisampling
                .set_multisampling_none()
            //no blending
                .disable_blending()
            //no depth testing
                .disable_depth_test()
            //connect the image format we will draw into, from draw image
                .set_color_attachment_format(scene_image.format)
                .set_depth_format(api::VK_FORMAT_UNDEFINED);

            //finally build the pipeline
            builder.build(&device)
        };

        device.destroy_shader_module(colored_tri_vert_sm);
        device.destroy_shader_module(colored_tri_frag_sm);

        return RenderSystem{
            device,
            swapchain,
            scene_image,
            frame_data,
            frame_index: 0,
            silly: 0,
            global_da,
            draw_image_dl,
            draw_image_ds,
            gradient_pl,
            gradient_p,
            triangle_pl,
            triangle_p,
        };
    }

    fn get_frame_data(&self) -> Rc<PerFrameData> {
        self.frame_data[self.swapchain.frame_index].clone()
    }

    fn draw_geometry(&self, cmd_buffer: &mut CommandBuffer) {
        let color_attachment = make_color_attachment_info(self.scene_image.view, None, api::VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL);

        //VkRenderingInfo renderInfo = vkinit::rendering_info(_drawExtent, &colorAttachment, nullptr);
        //vkCmdBeginRendering(cmd, &renderInfo);

        let draw_extent = api::VkExtent2D{ width: self.scene_image.dims.width, height: self.scene_image.dims.height };
        let render_info = make_rendering_info(draw_extent, &color_attachment, std::ptr::null());

        cmd_buffer.begin_rendering(render_info);

        cmd_buffer.bind_graphics_pipeline(self.triangle_p);
        cmd_buffer.set_viewport(draw_extent.width, draw_extent.height, 0, 0);
        cmd_buffer.set_scissor(draw_extent.width, draw_extent.height);
        cmd_buffer.draw(3, 1, 0, 0);

        cmd_buffer.end_rendering();
    }

    pub fn render(&mut self, _command_buffer: RenderCommandBuffer) {
        //todo: process incoming render commands

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

        self.silly += 1;
        let flash:  f32 = f32::sin((self.silly as f32) / 6000.0).abs();
        let flash2: f32 = f32::cos((self.silly as f32) / 6000.0).abs();
        let flash3: f32 = f32::tan((self.silly as f32) / 6000.0).abs();
        let clear_value = api::VkClearColorValue{ float32: [flash2, flash3, flash, 1.0] };

        let frame_data  = self.get_frame_data();
        let mut frame_state = frame_data.state.borrow_mut();

        let mut command_buffer = &mut frame_state.command_buffer;

        command_buffer.reset();
        command_buffer.begin_recording();

        command_buffer.transition_image(self.scene_image.image, api::VK_IMAGE_LAYOUT_UNDEFINED, api::VK_IMAGE_LAYOUT_GENERAL);

        { // Draw background
            //command_buffer.clear_color_image(self.scene_image.image, &clear_value);

            command_buffer.bind_compute_pipeline(self.gradient_p);

            let descriptors: [api::VkDescriptorSet; 1] = [ self.draw_image_ds ];
            command_buffer.bind_compute_descriptor_sets(self.gradient_pl, 0, descriptors.as_slice());

            let group_x = self.scene_image.dims.width  as f32 / 16.0;
            let group_y = self.scene_image.dims.height as f32 / 16.0;

            command_buffer.dispatch_compute(group_x.ceil() as u32, group_y.ceil() as u32, 1);
        }

        { // Draw geometry
           	command_buffer.transition_image(self.scene_image.image, api::VK_IMAGE_LAYOUT_GENERAL, api::VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL);
            self.draw_geometry(&mut command_buffer);
        }

        // Now, copy the scene framebuffer to the swapchain
        let swapchain_image  = self.swapchain.get_swapchain_image();
        let swapchain_extent = self.swapchain.get_extent();

        command_buffer.transition_image(self.scene_image.image, api::VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL, api::VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL);
        command_buffer.transition_image(swapchain_image,        api::VK_IMAGE_LAYOUT_UNDEFINED,                api::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL);

        let src_extent = api::VkExtent2D{ width: self.scene_image.dims.width, height: self.scene_image.dims.height };
        let dst_extent = api::VkExtent2D{ width: swapchain_extent.width,      height: swapchain_extent.height      };

        command_buffer.copy_image_to_image(self.scene_image.image, src_extent, swapchain_image, dst_extent);

        // Transition the swapchain image to present mode
        command_buffer.transition_image(swapchain_image, api::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, api::VK_IMAGE_LAYOUT_PRESENT_SRC_KHR);

        // End the Frame
        //

        command_buffer.end_recording();

        let cmd_buffer_si = command_buffer.get_submit_info();

        let render_sem  = self.swapchain.get_render_semaphore();
        let present_sem = self.swapchain.get_present_semaphore();

        // Want to wait on the PresentSemaphore, as that semaphore is signaled when the swapchain is ready
        let wait_info   = make_semaphore_submit_info(api::VK_PIPELINE_STAGE_2_COLOR_ATTACHMENT_OUTPUT_BIT_KHR, present_sem);
        // Will signal the renderSemaphore, to signal that rendering has finished
        let signal_info = make_semaphore_submit_info(api::VK_PIPELINE_STAGE_2_ALL_GRAPHICS_BIT, render_sem);

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

        self.device.destroy_pipeline(self.triangle_p);
        self.device.destroy_pipeline_layout(self.triangle_pl);

        self.device.destroy_pipeline(self.gradient_p);
        self.device.destroy_pipeline_layout(self.gradient_pl);

        self.device.destroy_descriptor_allocator(&mut self.global_da);
        self.device.destroy_descriptor_set_layout(self.draw_image_dl);

        for frame_data in &self.frame_data{
            self.device.destroy_command_pool(&mut frame_data.state.borrow_mut().command_pool);
        }

        self.device.destroy_image_memory(&mut self.scene_image);
        self.device.destroy_swapchain(&mut self.swapchain);
        self.device.destroy();
    }

    pub fn on_resize(&mut self, width: u32, height: u32)
    {
        self.swapchain.on_resize(width, height);
    }
}

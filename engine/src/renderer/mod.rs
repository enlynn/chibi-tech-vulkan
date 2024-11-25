mod graphics;

use graphics::*;
use graphics::{
    AllocatedImage,
    gpu_device::Device,
    gpu_swapchain::Swapchain,
    gpu_utils::*,
    gpu_command_pool::CommandPool,
    gpu_command_buffer::CommandBuffer,
};

use super::window::NativeSurface;

use std::borrow::BorrowMut;
use std::rc::Rc;
use std::cell::RefCell;

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
}

impl RenderSystem {
    pub fn new(create_info: RendererCreateInfo) -> RenderSystem {
        let device = Device::new(gpu_device::CreateInfo{
            features:         gpu_device::Features::default(),  //todo: make configurable
            surface:          create_info.surface,
            software_version: crate::make_app_version(0, 0, 1), //todo: make configurable
            software_name:    String::from("Testbed"),          //todo: make configurable
        });

        let swapchain = device.create_swapchain(None);

        let scene_image = {
            let image_usages: api::VkImageUsageFlags =
                api::VK_IMAGE_USAGE_TRANSFER_SRC_BIT |
                api::VK_IMAGE_USAGE_TRANSFER_DST_BIT |
                api::VK_IMAGE_USAGE_STORAGE_BIT      |
                api::VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT;

            device.allocate_image_memory(
                swapchain.get_extent(),
                api::VK_FORMAT_R16G16B16A16_SFLOAT,
                image_usages,
                api::VMA_MEMORY_USAGE_GPU_ONLY,
                api::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
                api::VK_IMAGE_ASPECT_COLOR_BIT)
        };

        let init_frame_data = |device: &Device| -> PerFrameData {
            let command_pool =   device.create_command_pool(QueueType::Graphics);
            let command_buffer = device.create_command_buffer(&command_pool);
            PerFrameData{ state: RefCell::new(PerFrameState { command_pool, command_buffer }) }
        };

        let mut frame_data = Vec::<Rc<PerFrameData>>::with_capacity(swapchain.images.len());
        for i in 0..swapchain.images.len() {
            frame_data.push(Rc::new(init_frame_data(&device)));
        }

        return RenderSystem{
            device,
            swapchain,
            scene_image,
            frame_data,
            frame_index: 0,
            silly: 0,
        };
    }

    fn get_frame_data(&self) -> Rc<PerFrameData> {
        self.frame_data[self.swapchain.frame_index].clone()
    }

    pub fn render(&mut self, _command_buffer: RenderCommandBuffer) {
        //todo: process incoming render commands

        //todo: process per-frame garbage

        // If the swapchain has been invalidated, recreate it.
        if !self.swapchain.is_valid()
        {
            self.device.wait_idle();

            self.swapchain = self.device.create_swapchain(Some(&self.swapchain));
            self.swapchain.validate();
            return; //don't render this frame...
        }

        if !self.swapchain.acquire_frame(&self.device) {
            return; // try again next frame
        }

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
        command_buffer.clear_color_image(self.scene_image.image, &clear_value);

        // Now, copy the scene framebuffer to the swapchain
        let swapchain_image = self.swapchain.get_swapchain_image();

        command_buffer.transition_image(self.scene_image.image, api::VK_IMAGE_LAYOUT_GENERAL,   api::VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL);
        command_buffer.transition_image(swapchain_image,        api::VK_IMAGE_LAYOUT_UNDEFINED, api::VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL);

        let swapchain_extent = self.swapchain.get_extent();
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

        for frame_data in &self.frame_data{
            self.device.destroy_command_pool(&mut frame_data.state.borrow_mut().command_pool);
        }

        self.device.destroy_image_memory(&mut self.scene_image);
        self.device.destroy_swapchain(&mut self.swapchain);
        self.device.destroy();
    }
}

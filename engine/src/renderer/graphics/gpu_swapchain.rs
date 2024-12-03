use vendor::vulkan::*;
use super::consts;
use super::gpu_utils::*;
use super::gpu_device::Device;

use std::ptr;
use std::rc::Rc;
use std::mem::MaybeUninit;

pub struct SwapchainFnTable {}

pub struct Swapchain {
    pub fns:                SwapchainFnTable,
    pub handle:             VkSwapchainKHR,
    pub present_queue:      VkQueue,

    // swapchain images
    pub image_views:        Vec<VkImageView>,
    pub images:             Vec<VkImage>,

    // synchronization state
    pub present_semaphores:     Vec<super::Semaphore>,
    pub render_semaphores:      Vec<super::Semaphore>,
    pub render_fences:          Vec<super::Fence>,
    pub frame_index:            usize,
    pub swapchain_index:        u32,

    pub cached_width:       u32,
    pub cached_height:      u32,
    pub known_generation:   usize,
    pub current_generation: usize,

}

impl Default for Swapchain {
    fn default() -> Self {
        Self {
            fns:                SwapchainFnTable {},
            handle:             ptr::null_mut(),
            present_queue:      ptr::null_mut(),
            image_views:        Vec::<VkImageView>::new(),
            images:             Vec::<VkImage>::new(),
            present_semaphores: Vec::<super::Semaphore>::new(),
            render_semaphores:  Vec::<super::Semaphore>::new(),
            render_fences:      Vec::<super::Fence>::new(),
            frame_index:        0,
            swapchain_index:    0,
            cached_width:       0,
            cached_height:      0,
            known_generation:   0,
            current_generation: 0,
        }
    }
}

impl Swapchain {
    pub fn validate(&mut self) {
        self.known_generation = self.current_generation;
    }

    pub fn invalidate(&mut self) {
        self.current_generation += 1;
    }

    pub fn is_valid(&self) -> bool {
        return self.current_generation == self.known_generation;
    }

    pub fn on_resize(&mut self, width: u32, height: u32) {
        self.cached_width  = width;
        self.cached_height = height;
        self.invalidate();
    }

    pub fn get_extent(&self) -> VkExtent3D {
        VkExtent3D{
            width:  self.cached_width,
            height: self.cached_height,
            depth:  1,
        }
    }

    pub fn get_swapchain_image(&self) -> VkImage {
        self.images[self.swapchain_index as usize]
    }

    pub fn get_swapchain_image_view(&self) -> VkImageView {
        self.image_views[self.swapchain_index as usize]
    }

    pub fn get_render_semaphore(&self) -> super::Semaphore {
        self.render_semaphores[self.frame_index as usize]
    }

    pub fn get_present_semaphore(&self) -> super::Semaphore {
        self.present_semaphores[self.frame_index as usize]
    }

    pub fn get_render_fence(&self) -> super::Fence {
        self.render_fences[self.frame_index as usize]
    }

    pub fn get_image_count(&self) -> usize {
        self.images.len()
    }

    pub fn acquire_frame(&mut self, device: &Device) -> bool {
        // Wait for the execution of the current frame to complete. The fence being free will allow this one to move on.
        //   Timeout of 1s
        let result = call_nothrow!(device.fns.wait_for_fences, device.handle, 1, &self.render_fences[self.frame_index], VK_TRUE, 1000000000);
        if result != VK_SUCCESS {
            println!("WARN :: begin_frame :: In-flight fence wait failure!");
            return false;
        }

        // Reset the fence for use on the next frame
        call_throw!(device.fns.reset_fences, device.handle, 1, &self.render_fences[self.frame_index]);

        // Acquire the next swapchain image. Timeout of 1s
        // mPresentSemaphore will be signaled when we are ready to render into the swapchain image.
        let mut swapchain_index: MaybeUninit<_> = MaybeUninit::<u32>::uninit();

        let result = call_nothrow!(device.fns.acquire_next_image, device.handle, self.handle, 1000000000,
            self.present_semaphores[self.frame_index], std::ptr::null_mut(), swapchain_index.as_mut_ptr());

        self.swapchain_index = unsafe { swapchain_index.assume_init() };

        if result == VK_ERROR_OUT_OF_DATE_KHR {
            self.invalidate();
            return false;
        } else if result != VK_SUCCESS && result != VK_SUBOPTIMAL_KHR {
            println!("ERROR :: begin_frame :: Failed to acquire swapchain image!");
            return false;
        }

        true
    }

    pub fn present_frame(&mut self, device: &Device) {
        // Return the image to the swapchain for presentation.
        let present_info = VkPresentInfoKHR{
            sType:              VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
            pNext:              std::ptr::null(),
            waitSemaphoreCount: 1,
            pWaitSemaphores:    &self.render_semaphores[self.frame_index], // Do not present until this semaphore has been signaled
            swapchainCount:     1,
            pSwapchains:        &self.handle,
            pImageIndices:      &self.swapchain_index,
            pResults:           std::ptr::null_mut(),
        };

        let result = call_nothrow!(device.fns.queue_present, self.present_queue, &present_info);
        if result == VK_ERROR_OUT_OF_DATE_KHR || result == VK_SUBOPTIMAL_KHR {
            // Swapchain is out of date, suboptimal or a framebuffer resize has occurred. Trigger swapchain recreation.
            println!("WARN :: present_frame :: vkQueuePresentKHR returned out of date or suboptimal.");

            self.invalidate();
        }
        else if (result != VK_SUCCESS)
        {
            panic!("Failed to present swap chain image!");
        }

        self.frame_index = (self.frame_index + 1) % self.render_fences.len();
    }
}

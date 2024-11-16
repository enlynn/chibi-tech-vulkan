
use crate::window::NativeSurface;

use super::api;
use super::consts;
use super::gpu_utils as util;

pub struct Features {}

pub struct CreateInfo {
    pub features: Features,
    pub surface: NativeSurface,
}

pub struct Instance {
    glb_fns:  util::GlobalFnTable,
    inst_fns: util::InstanceFnTable,
    handle:   api::VkInstance,
}

pub struct Gpu {
    handle: api::VkPhysicalDevice,
    //todo:
}

pub struct Display {
    //todo:
}

pub struct Surface {

}

pub struct SwapchainImage {

}

pub struct Swapchain {
    images: [SwapchainImage; consts::MAX_BUFFERED_FRAMES],
}

pub struct Device {
    global_fns: util::GlobalFnTable,

    //fns: util::DeviceFnTable

    //surface:   Surface,
    //swapchain: Swapchain,

    //gpus:     Vec<Rc<Gpu>>,
    //displays: Vec<Rc<Display>>,

    //gpu:     Rc<Gpu>,
    //display: Rc<Display>,
}

impl Device {
    pub fn new(_create_info: CreateInfo) -> Device {
        let global_fns: util::GlobalFnTable = match util::load_vulkan_proc_addr() {
            Ok(fns) => fns,
            Err(reason) => panic!("Failed to load vulkan library: {}", reason),
        };

        return Device{
            global_fns
        };
    }
}

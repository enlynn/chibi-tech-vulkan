
use super::consts;
use super::gpu_device::Device;

use std::rc::Rc;

#[derive(Copy, Clone)]
pub struct SwapchainImage {

}

pub struct SwapchainFnTable{

}

pub struct Swapchain {
    fns:    SwapchainFnTable,
    device: Rc<Device>,
    images: [SwapchainImage; consts::MAX_BUFFERED_FRAMES],
}

impl Default for SwapchainImage {
    fn default() -> Self { Self{} }
}

impl Swapchain {
    pub fn new(device: Rc<Device>) -> Self{
        return Self{
            fns:    SwapchainFnTable{},
            device,
            images: [SwapchainImage::default(); consts::MAX_BUFFERED_FRAMES],
        }
    }

    pub fn destroy(&mut self) {}

    pub fn on_resize(&mut self) {}

    pub fn begin_frame(&mut self) {}
    pub fn end_frame(&mut self) {}
}


use super::gpu_device::Device;

use std::rc::Rc;

struct GraphicsContext {}
struct ComputeContext  {}
struct TransferContext {}

pub struct DeviceContext {
    device:           Rc<Device>,

    graphics_context: GraphicsContext,
    compute_context:  ComputeContext,
    transfer_context: TransferContext,
}

impl DeviceContext {
    pub fn new(device: Rc<Device>) -> Self {
        Self{
            device,
            graphics_context: GraphicsContext{},
            compute_context:  ComputeContext{},
            transfer_context: TransferContext{},
        }
    }
}

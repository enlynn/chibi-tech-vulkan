use super::api;
use super::gpu_device as device;

/// Enabling validation layers will enable further error reporting from Vulkan, but can degrade performance.
pub const ENABLE_DEBUG_LAYER: bool = true;

pub const VK_API_VERSION: u32 = api::VK_API_VERSION_1_3;

pub const DEVICE_FEATURES: device::Features = device::Features{};

pub const VK_KHR_PORTABILITY_SUBSET_EXTENSION_NAME: &[u8; 26usize] = b"VK_KHR_portability_subset\0";
pub const VK_LAYER_KHRONOS_VALIDATION_LAYER_NAME: &[u8; 28usize] = b"VK_LAYER_KHRONOS_validation\0";


//std::array<const char*, 2> Device::cDeviceExtensions = {
//    VK_KHR_SWAPCHAIN_EXTENSION_NAME,
//    VK_KHR_TIMELINE_SEMAPHORE_EXTENSION_NAME,
//};
//
//std::array<const char*, 1> Device::cInstanceExtensions = {
//    VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME
//};

pub const MAX_BUFFERED_FRAMES: usize = 3;

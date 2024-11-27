
pub mod api;
pub mod consts;
pub mod gpu_utils;

pub mod gpu_device;
pub mod gpu_device_context;
pub mod gpu_swapchain;
pub mod gpu_command_pool;
pub mod gpu_command_buffer;
pub mod gpu_descriptors;
pub mod gpu_pipeline;

pub type Semaphore         = api::VkSemaphore;
pub type TimelineSemaphore = api::VkSemaphore;
pub type Fence             = api::VkFence;

pub struct AllocatedImage {
    pub image:  api::VkImage,
    pub view:   api::VkImageView,
    pub memory: api::VmaAllocation,
    pub dims:   api::VkExtent3D,
    pub format: api::VkFormat,
}

impl Default for AllocatedImage {
    fn default() -> Self {
        Self{
            image:  std::ptr::null_mut(),
            view:   std::ptr::null_mut(),
            memory: std::ptr::null_mut(),
            dims:   api::VkExtent3D::default(),
            format: api::VK_FORMAT_UNDEFINED,
        }
    }
}

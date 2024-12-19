use vendor::vulkan::*;

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

pub struct StructuredBuffer<T> {
    handle:          AllocatedBuffer,
    address:         VkDeviceAddress,
    capacity:        usize, // capacity per frame
    buffered_frames: usize, // total number of frames in the buffer

    // Mapped data
    mapped_frame:    *mut T,
    frame_index:     usize,
    count_in_frame:  usize,
}

impl<T> StructuredBuffer<T> {
    pub fn new(device: &Device, max_elements: usize, buffered_frames: usize) -> Self {
        let buffer_usage = VK_BUFFER_USAGE_STORAGE_BUFFER_BIT | VK_BUFFER_USAGE_SHADER_DEVICE_ADDRESS_BIT;
        let memory_usage = VMA_MEMORY_USAGE_CPU_TO_GPU;

        let size_per_frame = max_elements * std::mem::size_of::<T>();
        let total_size     = size_per_frame * buffered_frames;
        let buffer         = device.create_buffer(total_size, buffer_usage, memory_usage);
        let address        = device.get_buffer_device_address(&buffer);

        return Self{
            handle:         buffer,
            address,
            capacity:       max_elements,
            buffered_frames,
            mapped_frame:   std::ptr::null_mut(),
            frame_index:    0,
            count_in_frame: 0,
        }
    }

    pub fn destroy(&mut self, device: &Device) {
        device.destroy_buffer(&mut self.handle);
        self.address         = 0;
        self.capacity        = 0;
        self.buffered_frames = 0;
        self.mapped_frame    = std::ptr::null_mut();
        self.count_in_frame  = 0;
    }

    pub fn map_frame(&mut self, frame: usize) {
        assert!(frame < self.buffered_frames);

        let base_address = self.handle.get_allocation() as *mut T;
        let offset = self.capacity * frame;

        self.mapped_frame   = unsafe { base_address.add(offset) };
        self.frame_index    = frame;
        self.count_in_frame = 0;
    }

    pub fn unmap_frame(&mut self) {
        self.mapped_frame = std::ptr::null_mut();
    }

    pub fn write(&mut self, index: usize, val: T) {
        assert!(self.mapped_frame != std::ptr::null_mut());
        assert!(index < self.capacity);
        assert!(index < self.count_in_frame);

        let write_ptr = unsafe { self.mapped_frame.add(index) };
        unsafe { *write_ptr = val };
    }

    pub fn write_next(&mut self, val: T) -> u64 {
        assert!(self.mapped_frame != std::ptr::null_mut());
        assert!(self.count_in_frame + 1 < self.capacity);

        let result = self.count_in_frame;
        self.count_in_frame += 1;

        let write_ptr = unsafe { self.mapped_frame.add(result) };
        unsafe { *write_ptr = val };

        result as u64
    }

    // returns the device address for the bounded frame
    pub fn get_device_address(&self) -> VkDeviceAddress {
        assert!(self.mapped_frame != std::ptr::null_mut());
        return self.address + (self.capacity * self.frame_index * std::mem::size_of::<T>()) as VkDeviceAddress;
    }
}

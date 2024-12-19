use vendor::vulkan::VkFence;

use super::graphics::{
    gpu_command_buffer::CommandBuffer,
    gpu_device::Device,
    gpu_utils::{make_submit_info, QueueType}
};


// A function which takes the closure: fn func(cmd_buffer: &CommandBuffer)
pub fn immediate_submit<F>(device: &Device, command_buffer: &mut CommandBuffer, fence: VkFence, f: F) where
    F: Fn(&CommandBuffer)
{
    device.reset_fences(&fence);
    command_buffer.reset();
    command_buffer.begin_recording();

    // execute the function
    f(&command_buffer);

    command_buffer.end_recording();

    let cmd_buffer_si = command_buffer.get_submit_info();
    let submit = make_submit_info(cmd_buffer_si, None, None);

    device.queue_submit(QueueType::Graphics, submit, fence);
    device.wait_for_fences(fence);
}

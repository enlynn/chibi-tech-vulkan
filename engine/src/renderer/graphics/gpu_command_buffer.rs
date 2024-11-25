use crate::util::ffi::call;

use super::api::*;
use super::gpu_utils::*;

pub struct CommandBufferFnTable {
    pub begin_command_buffer:  FN_vkBeginCommandBuffer,
    pub end_command_buffer:    FN_vkEndCommandBuffer,
    pub reset_command_buffer:  FN_vkResetCommandBuffer,
    pub cmd_pipeline_barrier2: FN_vkCmdPipelineBarrier2,
    pub cmd_clear_color_image: FN_vkCmdClearColorImage,
    pub cmd_blit_image2:       FN_vkCmdBlitImage2,
}

#[derive(PartialEq)]
pub enum CommandBufferState {
    Closed,
    Open,
    Reset,
}

pub struct CommandBuffer {
    pub fns:    CommandBufferFnTable,
    pub handle: VkCommandBuffer,
    pub state:  CommandBufferState,
}

impl CommandBuffer {
    pub fn new(fns: CommandBufferFnTable, handle: VkCommandBuffer) -> Self {
        Self{
            fns, handle, state: CommandBufferState::Closed,
        }
    }

    pub fn begin_recording(&mut self) {
        assert!(self.state == CommandBufferState::Reset);

        let cmd_begin_info = make_command_buffer_begin_info(VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT);
        call_throw!(self.fns.begin_command_buffer, self.handle, &cmd_begin_info);

        self.state = CommandBufferState::Open;
    }

    pub fn end_recording(&mut self) {
        assert!(self.state == CommandBufferState::Open);

        call_throw!(self.fns.end_command_buffer, self.handle);
        self.state = CommandBufferState::Closed;
    }

    pub fn reset(&mut self) {
        assert!(self.state == CommandBufferState::Closed);

        call_throw!(self.fns.reset_command_buffer, self.handle, 0);
        self.state = CommandBufferState::Reset;
    }

    pub fn get_submit_info(&self) -> VkCommandBufferSubmitInfo {
        assert!(self.state == CommandBufferState::Closed);
        return make_command_buffer_submit_info(self.handle);
    }

    pub fn transition_image(&self, image: VkImage, current_layout: VkImageLayout, new_layout: VkImageLayout) {
        assert!(self.state == CommandBufferState::Open);

        let aspect_mask: VkImageAspectFlags = if new_layout == VK_IMAGE_LAYOUT_DEPTH_ATTACHMENT_OPTIMAL { VK_IMAGE_ASPECT_DEPTH_BIT } else { VK_IMAGE_ASPECT_COLOR_BIT };

        let barrier = VkImageMemoryBarrier2{
            sType:               VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER_2,
            pNext:               std::ptr::null(),
            srcStageMask:        VK_PIPELINE_STAGE_2_ALL_COMMANDS_BIT,
            srcAccessMask:       VK_ACCESS_2_MEMORY_WRITE_BIT,
            dstStageMask:        VK_PIPELINE_STAGE_2_ALL_COMMANDS_BIT,
            dstAccessMask:       VK_ACCESS_2_MEMORY_WRITE_BIT | VK_ACCESS_2_MEMORY_READ_BIT,
            oldLayout:           current_layout,
            newLayout:           new_layout,
            subresourceRange:    make_image_subresource_range(aspect_mask),
            image,
            srcQueueFamilyIndex: 0,
            dstQueueFamilyIndex: 0,
        };

        // note: can send multiple image barriers at once to improve performance
        let dep_info = VkDependencyInfo{
            sType:                    VK_STRUCTURE_TYPE_DEPENDENCY_INFO,
            pNext:                    std::ptr::null(),
            dependencyFlags:          0,
            memoryBarrierCount:       0,
            pMemoryBarriers:          std::ptr::null(),
            bufferMemoryBarrierCount: 0,
            pBufferMemoryBarriers:    std::ptr::null(),
            imageMemoryBarrierCount:  1,
            pImageMemoryBarriers:     &barrier,
        };

        call!(self.fns.cmd_pipeline_barrier2, self.handle, &dep_info);
    }

    pub fn clear_color_image(&self, image: VkImage, clear_value: &VkClearColorValue) {
        assert!(self.state == CommandBufferState::Open);

        let clear_range = make_image_subresource_range(VK_IMAGE_ASPECT_COLOR_BIT);
        call!(self.fns.cmd_clear_color_image, self.handle, image, VK_IMAGE_LAYOUT_GENERAL, clear_value, 1, &clear_range);
    }

    pub fn copy_image_to_image(&self,
        source:      VkImage, src_size: VkExtent2D,
        destination: VkImage, dst_size: VkExtent2D)
    {
        assert!(self.state == CommandBufferState::Open);

        let blit_region = VkImageBlit2{
            sType:          VK_STRUCTURE_TYPE_IMAGE_BLIT_2,
            pNext:          std::ptr::null(),
            srcSubresource: VkImageSubresourceLayers{
                aspectMask:     VK_IMAGE_ASPECT_COLOR_BIT,
                mipLevel:       0,
                baseArrayLayer: 0,
                layerCount:     1,
            },
            srcOffsets:     [
                VkOffset3D::default(),
                VkOffset3D{
                    x: src_size.width  as i32,
                    y: src_size.height as i32,
                    z: 1,
                },
            ],
            dstSubresource: VkImageSubresourceLayers{
                aspectMask:     VK_IMAGE_ASPECT_COLOR_BIT,
                mipLevel:       0,
                baseArrayLayer: 0,
                layerCount:     1,
            },
            dstOffsets:     [
                VkOffset3D::default(),
                VkOffset3D{
                    x: dst_size.width  as i32,
                    y: dst_size.height as i32,
                    z: 1,
                },
            ],
        };

        let blit_info = VkBlitImageInfo2{
            sType:          VK_STRUCTURE_TYPE_BLIT_IMAGE_INFO_2,
            pNext:          std::ptr::null(),
            srcImage:       source,
            srcImageLayout: VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
            dstImage:       destination,
            dstImageLayout: VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            regionCount:    1,
            pRegions:       &blit_region,
            filter:         VK_FILTER_LINEAR,
        };

        call!(self.fns.cmd_blit_image2, self.handle, &blit_info);
    }
}

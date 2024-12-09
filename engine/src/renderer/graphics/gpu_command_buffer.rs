use std::ffi::c_void;

use crate::util::ffi::call;

use super::{gpu_utils::*, AllocatedBuffer, AllocatedImage};

use vendor::vulkan::*;

pub struct CommandBufferFnTable {
    pub begin_command_buffer:     FN_vkBeginCommandBuffer,
    pub end_command_buffer:       FN_vkEndCommandBuffer,
    pub reset_command_buffer:     FN_vkResetCommandBuffer,
    pub cmd_pipeline_barrier2:    FN_vkCmdPipelineBarrier2,
    pub cmd_clear_color_image:    FN_vkCmdClearColorImage,
    pub cmd_blit_image2:          FN_vkCmdBlitImage2,
    pub cmd_bind_pipeline:        FN_vkCmdBindPipeline,
    pub cmd_bind_descriptor_sets: FN_vkCmdBindDescriptorSets,
    pub cmd_dispatch:             FN_vkCmdDispatch,
    pub cmd_begin_rendering:      FN_vkCmdBeginRendering,
    pub cmd_end_rendering:        FN_vkCmdEndRendering,
    pub cmd_set_scissor:          FN_vkCmdSetScissor,
    pub cmd_set_viewport:         FN_vkCmdSetViewport,
    pub cmd_draw:                 FN_vkCmdDraw,
    pub cmd_push_constants:       FN_vkCmdPushConstants,
    pub cmd_copy_buffer:          FN_vkCmdCopyBuffer,
    pub cmd_bind_index_buffer:    FN_vkCmdBindIndexBuffer,
    pub cmd_draw_indexed:         FN_vkCmdDrawIndexed,
    pub cmd_copy_buffer_to_image: FN_vkCmdCopyBufferToImage,
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

    // per-record transient state
    pub bound_pipeline: VkPipeline,
}

impl CommandBuffer {
    pub fn new(fns: CommandBufferFnTable, handle: VkCommandBuffer) -> Self {
        Self{
            fns, handle, state: CommandBufferState::Closed, bound_pipeline: std::ptr::null_mut(),
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

        self.state          = CommandBufferState::Reset;
        self.bound_pipeline = std::ptr::null_mut();
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

    pub fn bind_compute_pipeline(&mut self, pipeline: VkPipeline) {
        assert!(self.state == CommandBufferState::Open);

        if self.bound_pipeline != pipeline {
            self.bound_pipeline = pipeline;
        }

        call!(self.fns.cmd_bind_pipeline, self.handle, VK_PIPELINE_BIND_POINT_COMPUTE, pipeline);
    }

    pub fn bind_graphics_pipeline(&mut self, pipeline: VkPipeline) {
        assert!(self.state == CommandBufferState::Open);

        if self.bound_pipeline != pipeline {
            self.bound_pipeline = pipeline;
        }

        call!(self.fns.cmd_bind_pipeline, self.handle, VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline);
    }

    pub fn bind_compute_descriptor_sets(&mut self, pipeline_layout: VkPipelineLayout, first_set: u32, descriptor_sets: &[VkDescriptorSet]) {
        assert!(self.state == CommandBufferState::Open);

        //todo: dynamic descriptor sets

        call!(self.fns.cmd_bind_descriptor_sets, self.handle, VK_PIPELINE_BIND_POINT_COMPUTE, pipeline_layout, first_set,
            descriptor_sets.len() as u32, descriptor_sets.as_ptr(), 0, std::ptr::null());
    }

    pub fn bind_graphics_descriptor_sets(&mut self, pipeline_layout: VkPipelineLayout, first_set: u32, descriptor_sets: &[VkDescriptorSet]) {
        assert!(self.state == CommandBufferState::Open);

        //todo: dynamic descriptor sets

        call!(self.fns.cmd_bind_descriptor_sets, self.handle, VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline_layout, first_set,
            descriptor_sets.len() as u32, descriptor_sets.as_ptr(), 0, std::ptr::null());
    }

    pub fn bind_push_constants<PushConstants>(&mut self, pipeline_layout: VkPipelineLayout, stage: VkShaderStageFlagBits, push_consts: PushConstants, offset: u32) {
        assert!(self.state == CommandBufferState::Open);

        let consts_ptr: *const c_void = &push_consts as *const PushConstants as *const c_void;
        call!(self.fns.cmd_push_constants, self.handle, pipeline_layout, stage, offset, std::mem::size_of::<PushConstants>() as u32, consts_ptr);
    }

    pub fn dispatch_compute(&self, group_count_x: u32, group_count_y: u32, group_count_z: u32) {
        assert!(self.state == CommandBufferState::Open);
        call!(self.fns.cmd_dispatch, self.handle, group_count_x, group_count_y, group_count_z);
    }

    pub fn begin_rendering(&self, render_info: VkRenderingInfo) {
        assert!(self.state == CommandBufferState::Open);
        call!(self.fns.cmd_begin_rendering, self.handle, &render_info);
    }

    pub fn end_rendering(&self) {
        assert!(self.state == CommandBufferState::Open);
        call!(self.fns.cmd_end_rendering, self.handle);
    }

    pub fn set_viewport(&self, width: i32, height: i32, offset_x: u32, offset_y: u32) {
        assert!(self.state == CommandBufferState::Open);

        let viewport = VkViewport{
            x:        offset_x as f32,
            y:        offset_y as f32,
            width:    width    as f32,
            height:   height   as f32,
            minDepth: 0.0,
            maxDepth: 1.0,
        };

        call!(self.fns.cmd_set_viewport, self.handle, 0, 1, &viewport);
    }

    pub fn set_scissor(&self, width: u32, height: u32) {
        assert!(self.state == CommandBufferState::Open);

        let mut scissor = VkRect2D::default();
    	scissor.offset.x      = 0;
    	scissor.offset.y      = 0;
    	scissor.extent.width  = width;
    	scissor.extent.height = height;

    	call!(self.fns.cmd_set_scissor, self.handle, 0, 1, &scissor);
    }

    pub fn draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) {
        assert!(self.state == CommandBufferState::Open);
        call!(self.fns.cmd_draw, self.handle, vertex_count, instance_count, first_vertex, first_instance);
    }

    pub fn copy_buffer(&self, dst_buffer: &AllocatedBuffer, dst_offset: VkDeviceSize, src_buffer: &AllocatedBuffer, src_offset: VkDeviceSize, copy_size: VkDeviceSize) {
        assert!(self.state == CommandBufferState::Open);

        let copy_info = VkBufferCopy{
            srcOffset: src_offset,
            dstOffset: dst_offset,
            size:      copy_size,
        };

        call!(self.fns.cmd_copy_buffer, self.handle, src_buffer.buffer, dst_buffer.buffer, 1, &copy_info);
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

    pub fn copy_buffer_to_image(&self, upload_buffer: &AllocatedBuffer, dst_image: &AllocatedImage, size: VkExtent3D) {
        assert!(self.state == CommandBufferState::Open);

        let copy_region = VkBufferImageCopy{
            bufferOffset:      0,
            bufferRowLength:   0,
            bufferImageHeight: 0,
            imageSubresource:  VkImageSubresourceLayers{
                aspectMask:     VK_IMAGE_ASPECT_COLOR_BIT,
                mipLevel:       0,
                baseArrayLayer: 0,
                layerCount:     1,
            },
            imageOffset:       VkOffset3D::default(),
            imageExtent:       size,
        };

		call!(self.fns.cmd_copy_buffer_to_image, self.handle, upload_buffer.buffer, dst_image.image, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, 1, &copy_region);
    }

    pub fn bind_index_buffer(&self, index_buffer: &AllocatedBuffer) {
        call!(self.fns.cmd_bind_index_buffer, self.handle, index_buffer.buffer, 0, VK_INDEX_TYPE_UINT32);
    }

    pub fn draw_indexed(&self, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32) {
        call!(self.fns.cmd_draw_indexed, self.handle, index_count, instance_count, first_index, vertex_offset, first_instance);
    }

    pub fn generate_mipmaps(&self, image: &AllocatedImage) {
        assert!(self.state == CommandBufferState::Open);

        let mut image_size = VkExtent2D{ width: image.dims.width, height: image.dims.height };
        let mip_count      = ((image_size.width.max(image_size.height) as f32).log2().floor() as i32) + 1;

        for i in 0..mip_count {
            let half_size = VkExtent2D{ width: image_size.width / 2, height: image_size.height / 2 };

            let mut subresource_range = make_image_subresource_range(VK_IMAGE_ASPECT_COLOR_BIT);
            subresource_range.levelCount   = 1;
            subresource_range.baseMipLevel = i as u32;

            let image_barrier = VkImageMemoryBarrier2{
                sType:               VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER_2,
                pNext:               std::ptr::null(),
                srcStageMask:        VK_PIPELINE_STAGE_2_ALL_COMMANDS_BIT,
                srcAccessMask:       VK_ACCESS_2_MEMORY_WRITE_BIT,
                dstStageMask:        VK_PIPELINE_STAGE_2_ALL_COMMANDS_BIT,
                dstAccessMask:       VK_ACCESS_2_MEMORY_WRITE_BIT | VK_ACCESS_2_MEMORY_READ_BIT,
                oldLayout:           VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                newLayout:           VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
                srcQueueFamilyIndex: 0,
                dstQueueFamilyIndex: 0,
                image:               image.image,
                subresourceRange:    subresource_range,
            };

            let dep_info = VkDependencyInfo{
                sType:                    VK_STRUCTURE_TYPE_DEPENDENCY_INFO,
                pNext:                    std::ptr::null(),
                dependencyFlags:          0,
                memoryBarrierCount:       0,
                pMemoryBarriers:          std::ptr::null(),
                bufferMemoryBarrierCount: 0,
                pBufferMemoryBarriers:    std::ptr::null(),
                imageMemoryBarrierCount:  1,
                pImageMemoryBarriers:     &image_barrier,
            };

            call!(self.fns.cmd_pipeline_barrier2, self.handle, &dep_info);

            if i < mip_count - 1 {
                let blit_region = VkImageBlit2{
                    sType:          VK_STRUCTURE_TYPE_IMAGE_BLIT_2,
                    pNext:          std::ptr::null(),
                    srcSubresource: VkImageSubresourceLayers{
                        aspectMask:     VK_IMAGE_ASPECT_COLOR_BIT,
                        mipLevel:       i as u32,
                        baseArrayLayer: 0,
                        layerCount:     1,
                    },
                    srcOffsets:     [
                        VkOffset3D::default(),
                        VkOffset3D{
                            x: image_size.width  as i32,
                            y: image_size.height as i32,
                            z: 1,
                        },
                    ],
                    dstSubresource: VkImageSubresourceLayers{
                        aspectMask:     VK_IMAGE_ASPECT_COLOR_BIT,
                        mipLevel:       (i + 1) as u32,
                        baseArrayLayer: 0,
                        layerCount:     1,
                    },
                    dstOffsets:     [
                        VkOffset3D::default(),
                        VkOffset3D{
                            x: half_size.width  as i32,
                            y: half_size.height as i32,
                            z: 1,
                        },
                    ],
                };

                let blit_info = VkBlitImageInfo2{
                    sType:          VK_STRUCTURE_TYPE_BLIT_IMAGE_INFO_2,
                    pNext:          std::ptr::null(),
                    srcImage:       image.image,
                    srcImageLayout: VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL,
                    dstImage:       image.image,
                    dstImageLayout: VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                    regionCount:    1,
                    pRegions:       &blit_region,
                    filter:         VK_FILTER_LINEAR,
                };

                call!(self.fns.cmd_blit_image2, self.handle, &blit_info);

                image_size = half_size;
            }
        }

        // transition all mip levels into the final read_only layout
        self.transition_image(image.image, VK_IMAGE_LAYOUT_TRANSFER_SRC_OPTIMAL, VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL);
    }
}

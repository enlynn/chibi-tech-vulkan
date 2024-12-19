use vendor::vulkan::*;
use windows_sys::Win32::Foundation::ERROR_INVALID_TASK_INDEX;

use common::util::id::*;

use std::rc::Rc;
use std::ptr;

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

#[derive(Clone, Copy, PartialEq)]
pub struct TextureId(Id);
pub const INVALID_TEXTURE_ID: TextureId = TextureId(INVALID_ID);

pub enum TextureFormat {
    Unknown,
    R8g8b8a8Unorm,
    R8g8b8a8UnormSrgb,
    R8g8b8a8Bc1,
    R8g8b8a8Bc1Srgb,
}

pub enum TextureFlags {
    None      = 0x00,
    MipMapped = 0x01,
}

pub enum SamplerType {
    Linear,
    Nearest,
}

pub(crate) struct TextureCreateInfo {
    pub(crate) name:    String,        //todo: perhaps use a 128bit asset id
    pub(crate) format:  TextureFormat,
    pub(crate) flags:   TextureFlags,
    pub(crate) sampler: SamplerType,
    pub(crate) width:   u32,
    pub(crate) height:  u32,
    pub(crate) depth:   u32,
    pub(crate) pixels:  *const u8,
}

#[derive(Clone, Copy)]
pub(crate) struct Texture2D {
    pub(crate) image:   AllocatedImage,
    pub(crate) sampler: VkSampler,
}

#[derive(Clone, Copy)]
pub(crate) struct TextureMetadata {
    ref_count: u32,
    id:        TextureId,
}

const MAX_TEXTURES: usize = 100;

pub struct TextureSystem {
    device:                   Rc<Device>,
    metadata:                 [TextureMetadata; MAX_TEXTURES],
    textures:                 [Texture2D;       MAX_TEXTURES],
    id_gen:                   IdSystem,
    default_sampler_linear:   VkSampler,
	default_sampler_nearest:  VkSampler,
}

impl Default for TextureMetadata {
    fn default() -> Self {
        Self{
            ref_count: 0,
            id:        INVALID_TEXTURE_ID,
        }
    }
}

impl Default for Texture2D {
    fn default() -> Self {
        Self{
            image:   AllocatedImage::default(),
            sampler: std::ptr::null_mut(),
        }
    }
}

pub fn texture_format_to_vkformat(format: TextureFormat) -> VkFormat {
    match format {
        TextureFormat::Unknown           => return VK_FORMAT_UNDEFINED,
        TextureFormat::R8g8b8a8Unorm     => return VK_FORMAT_R8G8B8A8_UNORM,
        TextureFormat::R8g8b8a8UnormSrgb => return VK_FORMAT_R8G8B8A8_SRGB,
        TextureFormat::R8g8b8a8Bc1       => return VK_FORMAT_BC1_RGBA_UNORM_BLOCK,
        TextureFormat::R8g8b8a8Bc1Srgb   => return VK_FORMAT_BC1_RGBA_SRGB_BLOCK,
    }
}

pub fn get_bytes_per_pixel(format: VkFormat) -> u32 {
    match format {
        VK_FORMAT_UNDEFINED            => panic!("Unsupported vk format for textures: VK_FORMAT_UNDEFINED"),
        VK_FORMAT_R8G8B8A8_UNORM       => return 4,
        VK_FORMAT_R8G8B8A8_SRGB        => return 4,
        VK_FORMAT_BC1_RGBA_UNORM_BLOCK => return 4,
        VK_FORMAT_BC1_RGBA_SRGB_BLOCK  => return 4,
        default                        => panic!("Unsupported VkFormat"),
    }
}

impl TextureSystem {
    pub fn new(device: Rc<Device>) -> Self {
        let linear_sample  = device.create_sampler(VK_FILTER_LINEAR,  VK_FILTER_LINEAR);
        let nearest_sample = device.create_sampler(VK_FILTER_NEAREST, VK_FILTER_NEAREST);

        let mut result = Self{
            device,
            metadata: [TextureMetadata::default(); MAX_TEXTURES],
            textures: [Texture2D::default();       MAX_TEXTURES],
            id_gen:   IdSystem::new(MAX_TEXTURES),
            default_sampler_linear:  linear_sample,
            default_sampler_nearest: nearest_sample,
        };

        return result;
    }

    pub fn destroy(&mut self) {
        for i in 0..MAX_TEXTURES {
            if self.id_gen.is_id_valid(self.metadata[i].id.0) {
                self.device.destroy_image_memory(&mut self.textures[i].image);
                self.id_gen.free_id(self.metadata[i].id.0);
            }
        }

        self.device.destroy_sampler(self.default_sampler_linear);
        self.device.destroy_sampler(self.default_sampler_nearest);
    }

    pub fn create_texture(&mut self, info: TextureCreateInfo, mut command_buffer: &mut CommandBuffer, fence: VkFence) -> TextureId {
        let result: TextureId = TextureId(self.id_gen.alloc_id().unwrap_or(INVALID_TEXTURE_ID.0));
        if result == INVALID_TEXTURE_ID {
            return result;
        }

        let format          = texture_format_to_vkformat(info.format);
        let bytes_per_pixel = get_bytes_per_pixel(format);
        let data_size       = info.width * info.height * info.depth * bytes_per_pixel;
        let extent3d        = VkExtent3D{ width: info.width, height: info.height, depth: info.depth };

       	let mut upload_buffer = self.device.create_buffer(data_size as usize, VK_BUFFER_USAGE_TRANSFER_SRC_BIT, VMA_MEMORY_USAGE_CPU_TO_GPU);

        // copy the pixels into the upload buffer
        let upload_memory = upload_buffer.get_allocation();
        assert!(upload_memory != ptr::null_mut());

        let mut memory_as_bytes = upload_memory as *mut u8;
        unsafe { std::ptr::copy(info.pixels, memory_as_bytes, data_size as usize) };

        let usage = VK_IMAGE_USAGE_SAMPLED_BIT;
        let mipmapped = (info.flags as u32) & (TextureFlags::MipMapped as u32) != 0;
        let texture = self.device.allocate_image_memory(
            extent3d,
            format,
            usage | VK_IMAGE_USAGE_TRANSFER_DST_BIT | VK_IMAGE_USAGE_TRANSFER_SRC_BIT,
            VMA_MEMORY_USAGE_GPU_ONLY,
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            mipmapped
        );

        super::util::immediate_submit(&self.device, &mut command_buffer, fence,
            |command_buffer: &CommandBuffer| {
          		command_buffer.transition_image(texture.image, VK_IMAGE_LAYOUT_UNDEFINED, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL);
                command_buffer.copy_buffer_to_image(&upload_buffer, &texture, extent3d);

                if mipmapped {
                    command_buffer.generate_mipmaps(&texture);
                } else {
                    command_buffer.transition_image(texture.image, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL);
                }
            }
        );

        self.device.destroy_buffer(&mut upload_buffer);

        let texture_dst = &mut self.textures[result.0.get_index() as usize];
        let meta_dst    = &mut self.metadata[result.0.get_index() as usize];

        texture_dst.image   = texture;
        texture_dst.sampler = match info.sampler {
            SamplerType::Linear  => self.default_sampler_linear,
            SamplerType::Nearest => self.default_sampler_nearest,
        };

        meta_dst.id        = result;
        meta_dst.ref_count = 1;

        //todo: allow for texture lookup

        return result;
    }

    pub fn destroy_texture(&mut self, id: TextureId) {
        if self.id_gen.is_id_valid(id.0) && self.metadata[id.0.get_index() as usize].ref_count == 1 {
            self.device.destroy_image_memory(&mut self.textures[id.0.get_index() as usize].image);
            self.id_gen.free_id(id.0);
        }
    }

    pub fn get_texture_data(&self, id: TextureId) -> Option<Texture2D> {
        if self.id_gen.is_id_valid(id.0) {
            return Some(self.textures[id.0.get_index() as usize]);
        }

        return None;
    }
}

use vendor::vulkan::*;
use super::gpu_device::Device;

use std::{collections::VecDeque, ptr};

pub struct DescriptorLayoutBuilder {
    bindings: Vec<VkDescriptorSetLayoutBinding>,
}

pub enum DescriptorAllocatorFlags {
    None,
    AllowFree,
}

#[derive(Copy, Clone)]
pub struct PoolSizeRatio{
	pub descriptor_type: VkDescriptorType,
	pub ratio:           f32,
}

pub struct DescriptorAllocator {
    pub pool: VkDescriptorPool,
}

pub struct DescriptorAllocatorGrowable {
    ratios:        Vec<PoolSizeRatio>,
    sets_per_pool: u32,
    full_pools:    VecDeque<DescriptorAllocator>,
    ready_pools:   VecDeque<DescriptorAllocator>,
}

impl DescriptorLayoutBuilder {
    pub fn new() -> Self{
        Self { bindings: Vec::new() }
    }

    pub fn add_binding(&mut self, binding: u32, descriptor_type: VkDescriptorType) -> &mut Self {
        let binding = VkDescriptorSetLayoutBinding{
            binding,
            descriptorType:     descriptor_type,
            descriptorCount:    1,
            stageFlags:         0,
            pImmutableSamplers: std::ptr::null(),
        };

        self.bindings.push(binding);
        self
    }

    pub fn clear(&mut self) { self.bindings.clear(); }

    pub fn build(&mut self, device: &Device, stages: VkShaderStageFlags, flags: VkDescriptorSetLayoutCreateFlags) -> VkDescriptorSetLayout {
        for binding in &mut self.bindings {
            binding.stageFlags |= stages;
        }

        return device.create_descriptor_set_layout(self.bindings.as_slice(), flags);
    }
}

impl DescriptorAllocatorGrowable {
    pub fn new(device: &Device, ratios: &[PoolSizeRatio], sets_per_pool: u32) -> Self{
        let mut result = Self{
            ratios:      Vec::new(),
            sets_per_pool,
            full_pools:  VecDeque::new(),
            ready_pools: VecDeque::new(),
        };

        result.ratios.extend_from_slice(ratios);

        let pool = result.get_pool(&device);
        result.ready_pools.push_back(pool);

        result
    }

    fn get_pool(&mut self, device: &Device) -> DescriptorAllocator {
        if let Some(pool) = self.ready_pools.pop_back() {
            return pool;
        } else {
            let result = device.create_descriptor_allocator(self.sets_per_pool, DescriptorAllocatorFlags::None, self.ratios.as_slice());
            self.sets_per_pool = ((1.5 * self.sets_per_pool as f32) as u32).min(4092);
            return result;
        }
    }

    pub fn clear_pools(&mut self, device: &Device) {
        for pool in &self.ready_pools {
            device.clear_descriptor_allocator(pool);
        }

        for pool in &self.full_pools {
            device.clear_descriptor_allocator(pool);
        }

        self.ready_pools.append(&mut self.full_pools);
        self.full_pools.clear();
    }

    pub fn destroy_pools(&mut self, device: &Device) {
        for pool in &mut self.ready_pools {
            device.destroy_descriptor_allocator(pool);
        }

        for pool in &mut self.full_pools {
            device.destroy_descriptor_allocator(pool);
        }

        self.ready_pools.clear();
        self.full_pools.clear();
    }

    pub fn destroy(&mut self, device: &Device) {
        self.destroy_pools(device);
    }

    pub fn allocate(&mut self, device: &Device, layout: VkDescriptorSetLayout) -> VkDescriptorSet {
        let mut result: VkDescriptorSet = std::ptr::null_mut();

        let mut pool = self.get_pool(device);

        let ds = device.allocate_descriptors(&pool, layout);
        if let Some(set) = ds {
            result = set;
        } else {
            // current pool is full, so let's a get a new and try again.
            self.full_pools.push_back(pool);

            pool = self.get_pool(device);
            result = device.allocate_descriptors(&pool, layout).expect("Failed to allocate a descriptor set from the growable pool.");
        }

        self.ready_pools.push_back(pool);
        return result;
    }
}

pub struct DescriptorWriter {
    image_infos:  VecDeque<VkDescriptorImageInfo>,
    buffer_infos: VecDeque<VkDescriptorBufferInfo>,
    writes:       Vec<VkWriteDescriptorSet>,
}

impl DescriptorWriter {
    pub fn new() -> Self {
        Self{
            image_infos:  VecDeque::new(),
            buffer_infos: VecDeque::new(),
            writes:       Vec::new(),
        }
    }

    #[inline(always)]
    pub fn write_sampler(&mut self, binding: u32, sampler: VkSampler) {
        self.write_image(binding, std::ptr::null_mut(), sampler, 0, VK_DESCRIPTOR_TYPE_SAMPLER);
    }

    #[inline(always)]
    pub fn write_sampled_image(&mut self, binding: u32, image: VkImageView, layout: VkImageLayout) {
        self.write_image(binding, image, std::ptr::null_mut(), layout, VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE);
    }

    #[inline(always)]
    pub fn write_storage_image(&mut self, binding: u32, image: VkImageView, layout: VkImageLayout) {
        self.write_image(binding, image, std::ptr::null_mut(), layout, VK_DESCRIPTOR_TYPE_STORAGE_IMAGE);
    }

    #[inline(always)]
    pub fn write_combined_image_sampler(&mut self, binding: u32, image: VkImageView, sampler: VkSampler, layout: VkImageLayout) {
        self.write_image(binding, image, sampler, layout, VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER);
    }

    pub fn write_image(&mut self, binding: u32, image: VkImageView, sampler: VkSampler, layout: VkImageLayout, descriptor_type: VkDescriptorType) {
        // Allowed Descriptor Types:
        //   - VK_DESCRIPTOR_TYPE_SAMPLER                - requires the sampler
        //   - VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE          - requires image layout and view
        //   - VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER - requires all 3 set
        //   - VK_DESCRIPTOR_TYPE_STORAGE_IMAGE          - requires image layout and view, used for compute pipelines to access pixel data
        assert!(
            descriptor_type == VK_DESCRIPTOR_TYPE_SAMPLER                ||
            descriptor_type == VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE          ||
            descriptor_type == VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER ||
            descriptor_type == VK_DESCRIPTOR_TYPE_STORAGE_IMAGE,
            "Is not a valid descriptor type for an image",
        );

        self.image_infos.push_back(VkDescriptorImageInfo{ sampler, imageView: image, imageLayout: layout });

        let mut draw_write = VkWriteDescriptorSet::default();
        draw_write.dstBinding      = binding;
    	draw_write.descriptorCount = 1;
    	draw_write.descriptorType  = descriptor_type;
    	draw_write.pImageInfo      = self.image_infos.back().expect("There should be an element here -_-");

        self.writes.push(draw_write);
    }

    pub fn write_buffer(&mut self, binding: u32, buffer: VkBuffer, size: u64, offset: u64, descriptor_type: VkDescriptorType) {
        // Allowed Descriptor Types:
        //   - VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER
        //   - VK_DESCRIPTOR_TYPE_STORAGE_BUFFER
        //   - VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC
        //   - VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC
        assert!(
            descriptor_type == VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER         ||
            descriptor_type == VK_DESCRIPTOR_TYPE_STORAGE_BUFFER         ||
            descriptor_type == VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER_DYNAMIC ||
            descriptor_type == VK_DESCRIPTOR_TYPE_STORAGE_BUFFER_DYNAMIC,
            "Is not a valid descriptor type for a buffer."
        );

        self.buffer_infos.push_back(VkDescriptorBufferInfo{ buffer, offset, range:  size });

        let mut draw_write = VkWriteDescriptorSet::default();
        draw_write.dstBinding      = binding;
        draw_write.descriptorCount = 1;
        draw_write.descriptorType  = descriptor_type;
        draw_write.pBufferInfo     = self.buffer_infos.back().expect("There should be an element here -_-");

        self.writes.push(draw_write);
    }

    pub fn clear(&mut self) {
        self.writes.clear();
        self.buffer_infos.clear();
        self.image_infos.clear();
    }

    pub fn update_set(&mut self, device: &Device, set: VkDescriptorSet) {
        for write in &mut self.writes {
            write.dstSet = set;
        }

        device.update_descriptor_sets(&self.writes);
    }
}

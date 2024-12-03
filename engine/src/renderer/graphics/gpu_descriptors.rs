use vendor::vulkan::*;
use super::gpu_device::Device;

pub struct DescriptorLayoutBuilder {
    bindings: Vec<VkDescriptorSetLayoutBinding>,
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

pub enum DescriptorAllocatorFlags {
    None,
    AllowFree,
}

pub struct PoolSizeRatio{
	pub descriptor_type: VkDescriptorType,
	pub ratio:           f32,
}

pub struct DescriptorAllocator {
    pub pool: VkDescriptorPool,
}

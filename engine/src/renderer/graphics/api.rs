#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

include!(concat!(env!("OUT_DIR"), "/vulkan_bindings.rs"));

pub const VK_API_VERSION_1_1: u32 = 1u32 << 22u32 | 1u32 << 12u32;
pub const VK_API_VERSION_1_2: u32 = 1u32 << 22u32 | 2u32 << 12u32;
pub const VK_API_VERSION_1_3: u32 = 1u32 << 22u32 | 3u32 << 12u32;

use std::ptr;

impl Default for VkApplicationInfo {
    fn default() -> Self {
        Self{
            sType:              VK_STRUCTURE_TYPE_APPLICATION_INFO,
            pNext:              ptr::null_mut(),
            pApplicationName:   ptr::null_mut(),
            applicationVersion: 0,
            pEngineName:        ptr::null_mut(),
            engineVersion:      0,
            apiVersion:         VK_API_VERSION_1_3,
        }
    }
}

impl Default for VkInstanceCreateInfo {
    fn default() -> Self {
        Self{
            sType:                   VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
            pNext:                   ptr::null_mut(),
            flags:                   0,
            pApplicationInfo:        ptr::null_mut(),
            enabledLayerCount:       0,
            ppEnabledLayerNames:     ptr::null_mut(),
            enabledExtensionCount:   0,
            ppEnabledExtensionNames: ptr::null_mut(),
        }
    }
}

impl Default for VkDebugUtilsMessengerCreateInfoEXT {
    fn default() -> Self {
        Self{
            sType:           VK_STRUCTURE_TYPE_DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
            pNext:           ptr::null_mut(),
            flags:           0,
            messageSeverity: 0,
            messageType:     0,
            pfnUserCallback: None,
            pUserData:       ptr::null_mut(),
        }
    }
}

impl Default for VkLayerProperties {
    fn default() -> Self {
        Self{
            layerName:            [0; 256usize],
            specVersion:           0,
            implementationVersion: 0,
            description:          [0; 256usize],
        }
    }
}

impl Default for VkExtensionProperties {
    fn default() -> Self {
        Self {
            extensionName: [0; 256usize],
            specVersion:    0,
        }
    }
}

impl Default for VkQueueFamilyProperties {
    fn default() -> Self {
        Self{
            queueFlags:                  0,
            queueCount:                  0,
            timestampValidBits:          0,
            minImageTransferGranularity: VkExtent3D::default(),
        }
    }
}

impl Default for VkSurfaceFormatKHR {
    fn default() -> Self {
        Self{
            format:     VK_FORMAT_UNDEFINED,
            colorSpace: VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
        }
    }
}

impl Default for VkExtent2D {
    fn default() -> Self {
        Self{ width: 0, height: 0 }
    }
}

impl Default for VkExtent3D {
    fn default() -> Self {
        Self{ width: 0, height: 0, depth: 0 }
    }
}

impl Default for VkSurfaceCapabilitiesKHR {
    fn default() -> Self {
        Self {
            minImageCount:           0,
            maxImageCount:           0,
            currentExtent:           VkExtent2D::default(),
            minImageExtent:          VkExtent2D::default(),
            maxImageExtent:          VkExtent2D::default(),
            maxImageArrayLayers:     0,
            supportedTransforms:     0,
            currentTransform:        0,
            supportedCompositeAlpha: VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            supportedUsageFlags:     0,
        }
    }
}

impl Default for VkDeviceQueueCreateInfo {
    fn default() -> Self {
        Self{
            sType:            VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
            pNext:            ptr::null_mut(),
            flags:            0,
            queueFamilyIndex: 0,
            queueCount:       0,
            pQueuePriorities: ptr::null_mut(),
        }
    }
}

impl Default for VkPhysicalDeviceBufferDeviceAddressFeatures {
    fn default() -> Self {
        Self {
            sType:                            VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_BUFFER_DEVICE_ADDRESS_FEATURES,
            pNext:                            ptr::null_mut(),
            bufferDeviceAddress:              VK_FALSE,
            bufferDeviceAddressCaptureReplay: VK_FALSE,
            bufferDeviceAddressMultiDevice:   VK_FALSE,
        }
    }
}

impl Default for VkPhysicalDeviceSynchronization2Features {
    fn default() -> Self {
        Self{
            sType:            VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SYNCHRONIZATION_2_FEATURES,
            pNext:            ptr::null_mut(),
            synchronization2: VK_FALSE,
        }
    }
}

impl Default for VkPhysicalDeviceTimelineSemaphoreFeatures {
    fn default() -> Self {
        Self{
            sType:             VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_TIMELINE_SEMAPHORE_FEATURES,
            pNext:             ptr::null_mut(),
            timelineSemaphore: VK_FALSE,
        }
    }
}

impl Default for VkPhysicalDeviceFeatures {
    fn default() -> Self {
        Self{
            robustBufferAccess: VK_FALSE,
            fullDrawIndexUint32: VK_FALSE,
            imageCubeArray: VK_FALSE,
            independentBlend: VK_FALSE,
            geometryShader: VK_FALSE,
            tessellationShader: VK_FALSE,
            sampleRateShading: VK_FALSE,
            dualSrcBlend: VK_FALSE,
            logicOp: VK_FALSE,
            multiDrawIndirect: VK_FALSE,
            drawIndirectFirstInstance: VK_FALSE,
            depthClamp: VK_FALSE,
            depthBiasClamp: VK_FALSE,
            fillModeNonSolid: VK_FALSE,
            depthBounds: VK_FALSE,
            wideLines: VK_FALSE,
            largePoints: VK_FALSE,
            alphaToOne: VK_FALSE,
            multiViewport: VK_FALSE,
            samplerAnisotropy: VK_FALSE,
            textureCompressionETC2: VK_FALSE,
            textureCompressionASTC_LDR: VK_FALSE,
            textureCompressionBC: VK_FALSE,
            occlusionQueryPrecise: VK_FALSE,
            pipelineStatisticsQuery: VK_FALSE,
            vertexPipelineStoresAndAtomics: VK_FALSE,
            fragmentStoresAndAtomics: VK_FALSE,
            shaderTessellationAndGeometryPointSize: VK_FALSE,
            shaderImageGatherExtended: VK_FALSE,
            shaderStorageImageExtendedFormats: VK_FALSE,
            shaderStorageImageMultisample: VK_FALSE,
            shaderStorageImageReadWithoutFormat: VK_FALSE,
            shaderStorageImageWriteWithoutFormat: VK_FALSE,
            shaderUniformBufferArrayDynamicIndexing: VK_FALSE,
            shaderSampledImageArrayDynamicIndexing: VK_FALSE,
            shaderStorageBufferArrayDynamicIndexing: VK_FALSE,
            shaderStorageImageArrayDynamicIndexing: VK_FALSE,
            shaderClipDistance: VK_FALSE,
            shaderCullDistance: VK_FALSE,
            shaderFloat64: VK_FALSE,
            shaderInt64: VK_FALSE,
            shaderInt16: VK_FALSE,
            shaderResourceResidency: VK_FALSE,
            shaderResourceMinLod: VK_FALSE,
            sparseBinding: VK_FALSE,
            sparseResidencyBuffer: VK_FALSE,
            sparseResidencyImage2D: VK_FALSE,
            sparseResidencyImage3D: VK_FALSE,
            sparseResidency2Samples: VK_FALSE,
            sparseResidency4Samples: VK_FALSE,
            sparseResidency8Samples: VK_FALSE,
            sparseResidency16Samples: VK_FALSE,
            sparseResidencyAliased: VK_FALSE,
            variableMultisampleRate: VK_FALSE,
            inheritedQueries: VK_FALSE,
        }
    }
}

impl Default for VkPhysicalDeviceFeatures2 {
    fn default() -> Self {
        Self{
            sType:    VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_FEATURES_2,
            pNext:    ptr::null_mut(),
            features: VkPhysicalDeviceFeatures::default(),
        }
    }
}

impl Default for VkDeviceCreateInfo {
    fn default() -> Self {
        Self{
            sType:                   VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            pNext:                   ptr::null_mut(),
            flags:                   0,
            queueCreateInfoCount:    0,
            pQueueCreateInfos:       ptr::null_mut(),
            enabledLayerCount:       0,
            ppEnabledLayerNames:     ptr::null_mut(),
            enabledExtensionCount:   0,
            ppEnabledExtensionNames: ptr::null_mut(),
            pEnabledFeatures:        ptr::null_mut(),
        }
    }
}

impl Default for VkSwapchainCreateInfoKHR {
    fn default() -> Self {
        Self{
            sType:                 VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
            pNext:                 ptr::null_mut(),
            flags:                 0,
            surface:               ptr::null_mut(),
            minImageCount:         0,
            imageFormat:           VK_FORMAT_UNDEFINED,
            imageColorSpace:       VK_COLOR_SPACE_SRGB_NONLINEAR_KHR,
            imageExtent:           VkExtent2D::default(),
            imageArrayLayers:      0,
            imageUsage:            0,
            imageSharingMode:      VK_SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices:   ptr::null_mut(),
            preTransform:          0,
            compositeAlpha:        VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            presentMode:           VK_PRESENT_MODE_IMMEDIATE_KHR,
            clipped:               VK_FALSE,
            oldSwapchain:          ptr::null_mut(),
        }
    }
}

impl Default for VkComponentMapping {
    fn default() -> Self{
        Self{
            r: VK_COMPONENT_SWIZZLE_IDENTITY,
            g: VK_COMPONENT_SWIZZLE_IDENTITY,
            b: VK_COMPONENT_SWIZZLE_IDENTITY,
            a: VK_COMPONENT_SWIZZLE_IDENTITY,
        }
    }
}

impl Default for VkImageSubresourceRange{
    fn default() -> Self {
        Self{
            aspectMask:     0,
            baseMipLevel:   0,
            levelCount:     0,
            baseArrayLayer: 0,
            layerCount:     0,
        }
    }
}

impl Default for VkImageViewCreateInfo {
    fn default() -> Self {
        Self{
            sType:            VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
            pNext:            ptr::null_mut(),
            flags:            0,
            image:            ptr::null_mut(),
            viewType:         VK_IMAGE_VIEW_TYPE_2D,
            format:           VK_FORMAT_UNDEFINED,
            components:       VkComponentMapping::default(),
            subresourceRange: VkImageSubresourceRange::default(),
        }
    }
}

impl Default for VkCommandPoolCreateInfo {
    fn default() -> Self {
        Self{
            sType:            VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            pNext:            ptr::null(),
            flags:            VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
            queueFamilyIndex: 0,
        }
    }
}

impl Default for VkCommandBufferAllocateInfo {
    fn default() -> Self {
        Self{
            sType:              VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext:              ptr::null(),
            commandPool:        ptr::null_mut(),
            level:              VK_COMMAND_BUFFER_LEVEL_PRIMARY,
            commandBufferCount: 0,
        }
    }
}

impl Default for VkSemaphoreCreateInfo {
    fn default() -> Self {
        Self {
            sType: VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
        }
    }
}

impl Default for VkSemaphoreTypeCreateInfo {
    fn default() -> Self {
        Self{
            sType:         VK_STRUCTURE_TYPE_SEMAPHORE_TYPE_CREATE_INFO_KHR,
            pNext:         ptr::null(),
            semaphoreType: VK_SEMAPHORE_TYPE_TIMELINE_KHR,
            initialValue:  0,
        }
    }
}

impl Default for VkFenceCreateInfo {
    fn default() -> Self {
        Self{
            sType: VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: ptr::null(),
            flags: 0,
        }
    }
}

impl Default for VmaAllocatorCreateInfo {
    fn default() -> Self {
        Self{
            flags:                          0,
            physicalDevice:                 ptr::null_mut(),
            device:                         ptr::null_mut(),
            preferredLargeHeapBlockSize:    0,
            pAllocationCallbacks:           ptr::null_mut(),
            pDeviceMemoryCallbacks:         ptr::null_mut(),
            pHeapSizeLimit:                 ptr::null_mut(),
            pVulkanFunctions:               ptr::null_mut(),
            instance:                       ptr::null_mut(),
            vulkanApiVersion:               VK_API_VERSION_1_3,
            pTypeExternalMemoryHandleTypes: ptr::null_mut(),
        }
    }
}

impl Default for VmaAllocationCreateInfo {
    fn default() -> Self{
        Self{
            flags:          0,
            usage:          0,
            requiredFlags:  0,
            preferredFlags: 0,
            memoryTypeBits: 0,
            pool:           ptr::null_mut(),
            pUserData:      ptr::null_mut(),
            priority:       0.0,
        }
    }
}

impl Default for VkOffset3D {
    fn default() -> Self {
        Self{
            x: 0,
            y: 0,
            z: 0,
        }
    }
}

impl Default for VkDescriptorImageInfo {
    fn default() -> Self {
        Self{
            sampler:     ptr::null_mut(),
            imageView:   ptr::null_mut(),
            imageLayout: VK_IMAGE_LAYOUT_UNDEFINED,
        }
    }
}

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

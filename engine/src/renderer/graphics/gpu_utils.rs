use std::ptr;

use crate::core::os;
use crate::util::ffi::*;

use vendor::vulkan::*;
use super::consts;

/* ======================================================================== */
/* Helpful macros                                                           */

macro_rules! call_throw {
    ($call:expr, $($arg:expr),*) => {{
        let result = unsafe { ($call)($($arg,)*) };
        if result < 0 {
            panic!("{} failed: {}", stringify!(call), result);
        }
    }};
}

macro_rules! call_nothrow {
    ($call:expr, $($arg:expr),*) => {{
        let result = unsafe { ($call)($($arg,)*) };
        result
    }};
}

pub(crate) use call_throw;
pub(crate) use call_nothrow;

macro_rules! get_vk_procaddr_optional {
    ($call:expr, $obj:expr, $func:ident) => {{
        let pfn = unsafe { ($call)($obj, cstr_stringify!($func)) };
        let val: concat_idents!(PFN_, $func) = unsafe { std::mem::transmute_copy(&pfn) };
        val
    }};
}

macro_rules! get_vk_procaddr {
    ($call:expr, $obj:expr, $func:ident) => {{
        let pfn = unsafe { ($call)($obj, cstr_stringify!($func)) };
        if pfn.is_none() {
            panic!("Couldn't import {}", stringify!($func));
        }
        let val: concat_idents!(FN_, $func) = unsafe { std::mem::transmute_copy(&pfn) };
        val
    }};
}

macro_rules! get_device_procaddr {
        ($name:ident) => {
        get_vk_procaddr!(get_device_procaddr, device, $name)
    };
}

macro_rules! redef {
    ($name:ident) => {
        pub const $name: u32 = concat_idents!(VK_, $name);
    };
}


/* ======================================================================== */
/* Vulkan Function Tables                                                   */

pub struct GlobalFnTable {
    dll: os::DllLibrary,

    pub create_instance:                     FN_vkCreateInstance,
    pub get_inst_procaddr:                   FN_vkGetInstanceProcAddr,
    enumerate_instance_extension_properties: FN_vkEnumerateInstanceExtensionProperties,
    enumerate_instance_layer_properties:     FN_vkEnumerateInstanceLayerProperties,
}

impl GlobalFnTable {
    pub fn enumerate_instance_extensions(&self) -> Vec<VkExtensionProperties> {
        let mut extension_count: u32 = 0;
        call_throw!(self.enumerate_instance_extension_properties, ptr::null(), &mut extension_count as *mut u32, ptr::null_mut());

        if extension_count > 0 {
            let mut extensions = Vec::<VkExtensionProperties>::with_capacity(extension_count as usize);
            extensions.resize(extension_count as usize, VkExtensionProperties::default());

            call_throw!(self.enumerate_instance_extension_properties, ptr::null(), &mut extension_count as *mut u32, extensions.as_mut_ptr());
            return extensions;
        }
        else
        {
            return Vec::<VkExtensionProperties>::with_capacity(0);
        }
    }

    pub fn enumerate_instance_layers(&self) -> Vec<VkLayerProperties> {
        let mut layer_count: u32 = 0;
        call_throw!(self.enumerate_instance_layer_properties, &mut layer_count as *mut u32, ptr::null_mut());

        if layer_count > 0 {
            let mut layers = Vec::<VkLayerProperties>::with_capacity(layer_count as usize);
            layers.resize(layer_count as usize, VkLayerProperties::default());

            call_throw!(self.enumerate_instance_layer_properties, &mut layer_count as *mut u32, layers.as_mut_ptr());
            return layers;
        }
        else
        {
            return Vec::<VkLayerProperties>::with_capacity(0);
        }
    }
}

pub struct InstanceFnTable
{
    pub(crate) get_device_procaddr:          FN_vkGetDeviceProcAddr,
    pub(crate) create_device:                FN_vkCreateDevice,
    pub(crate) destroy_instance:             FN_vkDestroyInstance,
    pub(crate) destroy_surface:              FN_vkDestroySurfaceKHR,

    enum_gpu_ext_props:                      FN_vkEnumerateDeviceExtensionProperties,
    enum_physical_devices:                   FN_vkEnumeratePhysicalDevices,

    pub(crate) get_gpu_memory_properties:    FN_vkGetPhysicalDeviceMemoryProperties,
    pub(crate) get_gpu_memory_properties2:   FN_vkGetPhysicalDeviceMemoryProperties2,
    pub(crate) get_gpu_properties:           FN_vkGetPhysicalDeviceProperties,
    pub(crate) get_gpu_features:             FN_vkGetPhysicalDeviceFeatures,

    pub(crate) get_gpu_surface_capabilities: FN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR,
    pub(crate) get_gpu_surface_support:      FN_vkGetPhysicalDeviceSurfaceSupportKHR,
    pub(crate) get_gpu_format_properties:    FN_vkGetPhysicalDeviceFormatProperties,

    get_gpu_surface_formats:                 FN_vkGetPhysicalDeviceSurfaceFormatsKHR,
    get_gpu_queue_family_properties:         FN_vkGetPhysicalDeviceQueueFamilyProperties,
    get_gpu_surface_present_modes:           FN_vkGetPhysicalDeviceSurfacePresentModesKHR,

    #[cfg(target_os = "linux")]
    pub(crate) create_wayland_surface:       PFN_vkCreateWaylandSurfaceKHR,
    #[cfg(target_os = "linux")]
    pub(crate) create_xlib_surface:          PFN_vkCreateXlibSurfaceKHR,
    #[cfg(target_os = "windows")]
    pub(crate) create_win32_surface:         PFN_vkCreateWin32SurfaceKHR,

    pub(crate) create_debug_messenger:       PFN_vkCreateDebugUtilsMessengerEXT,         // note(enlynn): don't need to *always* load this function
    pub(crate) destroy_debug_messenger:      PFN_vkDestroyDebugUtilsMessengerEXT,
}

impl InstanceFnTable {
    pub fn enumerate_gpu_present_modes(&self, gpu: VkPhysicalDevice, surface: VkSurfaceKHR) -> Vec<VkPresentModeKHR> {
        let mut present_mode_count: u32 = 0;
        call_throw!(self.get_gpu_surface_present_modes, gpu, surface, &mut present_mode_count, ptr::null_mut());

        if present_mode_count > 0 {
            let mut present_mdoes = Vec::<VkPresentModeKHR>::with_capacity(present_mode_count as usize);
            present_mdoes.resize(present_mode_count as usize, VkPresentModeKHR::default());
            call_throw!(self.get_gpu_surface_present_modes, gpu, surface, &mut present_mode_count, present_mdoes.as_mut_ptr());

            return present_mdoes;
        } else {
            return Vec::with_capacity(0);
        }
    }

    pub fn enumerate_gpu_surface_formats(&self, gpu: VkPhysicalDevice, surface: VkSurfaceKHR) -> Vec<VkSurfaceFormatKHR> {
        let mut format_count: u32 = 0;
        call_throw!(self.get_gpu_surface_formats, gpu, surface, &mut format_count, ptr::null_mut());

        if format_count > 0 {
            let mut formats = Vec::<VkSurfaceFormatKHR>::with_capacity(format_count as usize);
            formats.resize(format_count as usize, VkSurfaceFormatKHR::default());
            call_throw!(self.get_gpu_surface_formats, gpu, surface, &mut format_count, formats.as_mut_ptr());

            return formats;
        } else {
            return Vec::with_capacity(0);
        }
    }

    pub fn enumerate_gpu_queue_family_properties(&self, gpu: VkPhysicalDevice) -> Vec<VkQueueFamilyProperties> {
        let mut queue_count: u32 = 0;
        call!(self.get_gpu_queue_family_properties, gpu, &mut queue_count as *mut u32, ptr::null_mut());

        if queue_count > 0 {
            let mut properties = Vec::<VkQueueFamilyProperties>::with_capacity(queue_count as usize);
            properties.resize(queue_count as usize, VkQueueFamilyProperties::default());
            call!(self.get_gpu_queue_family_properties, gpu, &mut queue_count as *mut u32, properties.as_mut_ptr());

            return properties;
        } else {
            return Vec::with_capacity(0);
        }
    }

    pub fn enumerate_device_extensions(&self, gpu: VkPhysicalDevice) -> Vec<VkExtensionProperties> {
        let mut ext_count: u32 = 0;
        call_throw!(self.enum_gpu_ext_props, gpu, ptr::null_mut(), &mut ext_count as *mut u32, ptr::null_mut());

        if ext_count > 0 {
            let mut extensions = Vec::<VkExtensionProperties>::with_capacity(ext_count as usize);
            extensions.resize(ext_count as usize, VkExtensionProperties::default());
            call_throw!(self.enum_gpu_ext_props, gpu, ptr::null_mut(), &mut ext_count as *mut u32, extensions.as_mut_ptr());

            return extensions;
        } else {
            return Vec::with_capacity(0);
        }
    }

    pub fn enumerate_gpus(&self, instance: VkInstance) -> Vec<VkPhysicalDevice> {
        let mut device_count: u32 = 0;
        call_throw!(self.enum_physical_devices, instance, &mut device_count as *mut u32, ptr::null_mut());

        if device_count > 0 {
            let mut gpus = Vec::<VkPhysicalDevice>::with_capacity(device_count as usize);
            gpus.resize(device_count as usize, std::ptr::null_mut());

            call_throw!(self.enum_physical_devices, instance, &mut device_count as *mut u32, gpus.as_mut_ptr());

            return gpus;
        } else {
            return Vec::with_capacity(0);
        }
    }
}

pub struct DeviceFnTable
{
    pub acquire_next_image:           FN_vkAcquireNextImageKHR,
    pub alloc_command_buffers:        FN_vkAllocateCommandBuffers,
    pub alloc_descriptor_sets:        FN_vkAllocateDescriptorSets,
    pub alloc_memory:                 FN_vkAllocateMemory,
    pub begin_command_buffer:         FN_vkBeginCommandBuffer,
    pub bind_buffer_memory:           FN_vkBindBufferMemory,
    pub cmd_begin_render_pass:        FN_vkCmdBeginRenderPass,
    pub cmd_copy_buffer:              FN_vkCmdCopyBuffer,
    pub cmd_end_render_pass:          FN_vkCmdEndRenderPass,
    pub cmd_pipeline_barrier:         FN_vkCmdPipelineBarrier,
    pub create_buffer:                FN_vkCreateBuffer,
    pub create_command_pool:          FN_vkCreateCommandPool,
    pub create_descriptor_pool:       FN_vkCreateDescriptorPool,
    pub create_descriptor_set_layout: FN_vkCreateDescriptorSetLayout,
    pub create_image:                 FN_vkCreateImage,
    pub create_image_view:            FN_vkCreateImageView,
    pub create_fence:                 FN_vkCreateFence,
    pub create_framebuffer:           FN_vkCreateFramebuffer,
    pub create_pipeline_cache:        FN_vkCreatePipelineCache,
    pub create_pipeline_layout:       FN_vkCreatePipelineLayout,
    pub create_render_pass:           FN_vkCreateRenderPass,
    pub create_sampler:               FN_vkCreateSampler,
    pub create_semaphore:             FN_vkCreateSemaphore,
    pub create_shader_module:         FN_vkCreateShaderModule,
    pub create_swapchain:             FN_vkCreateSwapchainKHR,
    pub destroy_buffer:               FN_vkDestroyBuffer,
    pub destroy_command_pool:         FN_vkDestroyCommandPool,
    pub destroy_device:               FN_vkDestroyDevice,
    pub destroy_fence:                FN_vkDestroyFence,
    pub destroy_framebuffer:          FN_vkDestroyFramebuffer,
    pub destroy_image:                FN_vkDestroyImage,
    pub destroy_image_view:           FN_vkDestroyImageView,
    pub destroy_semaphore:            FN_vkDestroySemaphore,
    pub destroy_render_pass:          FN_vkDestroyRenderPass,
    pub destroy_swapchain:            FN_vkDestroySwapchainKHR,
    pub wait_idle:                    FN_vkDeviceWaitIdle,
    pub end_command_buffer:           FN_vkEndCommandBuffer,
    pub flush_mapped_memory_ranges:   FN_vkFlushMappedMemoryRanges,
    pub free_command_buffers:         FN_vkFreeCommandBuffers,
    pub free_memory:                  FN_vkFreeMemory,
    pub get_buffer_memory_reqs:       FN_vkGetBufferMemoryRequirements,
    pub get_queue:                    FN_vkGetDeviceQueue,
    pub get_swapchain_images:         FN_vkGetSwapchainImagesKHR,
    pub map_memory:                   FN_vkMapMemory,
    pub queue_present:                FN_vkQueuePresentKHR,
    pub queue_submit:                 FN_vkQueueSubmit,
    pub queue_submit2:                FN_vkQueueSubmit2,
    pub reset_command_buffer:         FN_vkResetCommandBuffer,
    pub reset_command_pool:           FN_vkResetCommandPool,
    pub reset_fences:                 FN_vkResetFences,
    pub unmap_memory:                 FN_vkUnmapMemory,
    pub update_descriptor_sets:       FN_vkUpdateDescriptorSets,
    pub wait_for_fences:              FN_vkWaitForFences,

    pub invalidate_mapped_memory_ranges: FN_vkInvalidateMappedMemoryRanges,
    pub bind_image_memory:               FN_vkBindImageMemory,
    pub get_image_memory_reqs:           FN_vkGetImageMemoryRequirements,
    pub get_buffer_memory_reqs2:         FN_vkGetBufferMemoryRequirements2,
    pub get_image_memory_reqs2:          FN_vkGetImageMemoryRequirements2,
    pub bind_buffer_memory2:             FN_vkBindBufferMemory2,
    pub bind_image_memory2:              FN_vkBindImageMemory2,
    pub get_device_buffer_memory_reqs:   FN_vkGetDeviceBufferMemoryRequirements,
    pub get_device_image_memory_reqs:    FN_vkGetDeviceImageMemoryRequirements,

    pub cmd_pipeline_barrier2:           FN_vkCmdPipelineBarrier2,
    pub cmd_clear_color_image:           FN_vkCmdClearColorImage,
    pub cmd_blit_image2:                 FN_vkCmdBlitImage2,

    pub reset_descriptor_pool:           FN_vkResetDescriptorPool,
    pub destroy_descriptor_pool:         FN_vkDestroyDescriptorPool,
    pub destroy_descriptor_set_layout:   FN_vkDestroyDescriptorSetLayout,
    pub destroy_shader_module:           FN_vkDestroyShaderModule,
    pub destroy_pipeline_layout:         FN_vkDestroyPipelineLayout,
    pub create_compute_pipeline:         FN_vkCreateComputePipelines,
    pub create_graphics_pipeline:        FN_vkCreateGraphicsPipelines,
    pub destroy_pipeline:                FN_vkDestroyPipeline,
    pub cmd_bind_pipeline:               FN_vkCmdBindPipeline,
    pub cmd_bind_descriptor_sets:        FN_vkCmdBindDescriptorSets,
    pub cmd_dispatch:                    FN_vkCmdDispatch,
    pub cmd_begin_rendering:             FN_vkCmdBeginRendering,
    pub cmd_end_rendering:               FN_vkCmdEndRendering,
    pub cmd_set_scissor:                 FN_vkCmdSetScissor,
    pub cmd_set_viewport:                FN_vkCmdSetViewport,
    pub cmd_draw:                        FN_vkCmdDraw,
    pub cmd_push_constants:              FN_vkCmdPushConstants,
    pub get_device_address:              FN_vkGetBufferDeviceAddress,
    pub cmd_bind_index_buffer:           FN_vkCmdBindIndexBuffer,
    pub cmd_draw_indexed:                FN_vkCmdDrawIndexed,
    pub cmd_copy_buffer_to_image:        FN_vkCmdCopyBufferToImage,
    pub destroy_sampler:                 FN_vkDestroySampler,
}

/* ======================================================================== */
/* Vulkan Helper get_gpu_featuresFunctions                                  */

pub fn load_vulkan_proc_addr() -> Result<GlobalFnTable, String> {
    let lib: os::DllLibrary = if cfg!(unix) {
        let mut lib = os::DllLibrary::load("libvulkan.so\0");
        if lib.is_none() {
            lib = os::DllLibrary::load("libvulkan.so.1\0");
        }
        if let Some(valid_lib) = lib {
            valid_lib
        } else {
            return Err("Vulkan not found".to_string());
        }
    } else {
        return Err("Unsupported operating system".to_string());
    };

    let get_inst_procaddr: FN_vkGetInstanceProcAddr = unsafe { lib.get_fn("vkGetInstanceProcAddr\0").unwrap() };

    macro_rules! get_inst_procaddr {
        ($inst:expr, $name:ident) => {
            get_vk_procaddr!(get_inst_procaddr, $inst, $name)
        };
    }

    macro_rules! get_inst_procaddr_optional {
        ($inst:expr, $name:ident) => {
            get_vk_procaddr_optional!(get_inst_procaddr, $inst, $name)
        };
    }

    let create_instance  = get_vk_procaddr!(get_inst_procaddr, ptr::null_mut(), vkCreateInstance);
    let enum_ext_props   = get_inst_procaddr!(ptr::null_mut(), vkEnumerateInstanceExtensionProperties);
    let enum_layer_props = get_inst_procaddr!(ptr::null_mut(), vkEnumerateInstanceLayerProperties);

    return Ok(GlobalFnTable {
        dll: lib,
        get_inst_procaddr,
        create_instance,
        enumerate_instance_extension_properties: enum_ext_props,
        enumerate_instance_layer_properties:     enum_layer_props,
    })
}

pub fn load_instance_functions(gbl: &GlobalFnTable, inst: VkInstance) -> Result<InstanceFnTable, String> {
    macro_rules! get_inst_procaddr {
        ($inst:expr, $name:ident) => {
            get_vk_procaddr!(gbl.get_inst_procaddr, $inst, $name)
        };
    }

    macro_rules! get_inst_procaddr_optional {
        ($inst:expr, $name:ident) => {
            get_vk_procaddr_optional!(gbl.get_inst_procaddr, $inst, $name)
        };
    }

    let create_debug_messenger = if consts::ENABLE_DEBUG_LAYER {
        Some(get_inst_procaddr!(inst, vkCreateDebugUtilsMessengerEXT))
    } else {
        None
    };

    let destroy_debug_messenger = if consts::ENABLE_DEBUG_LAYER {
        Some(get_inst_procaddr!(inst, vkDestroyDebugUtilsMessengerEXT))
    } else {
        None
    };

    let get_device_procaddr: FN_vkGetDeviceProcAddr = get_vk_procaddr!(gbl.get_inst_procaddr, inst, vkGetDeviceProcAddr);

    let funcs = InstanceFnTable {
        get_device_procaddr,
        create_device:                   get_inst_procaddr!(inst, vkCreateDevice),
        destroy_instance:                get_inst_procaddr!(inst, vkDestroyInstance),
        destroy_surface:                 get_inst_procaddr!(inst, vkDestroySurfaceKHR),
        enum_gpu_ext_props:              get_inst_procaddr!(inst, vkEnumerateDeviceExtensionProperties),
        enum_physical_devices:           get_inst_procaddr!(inst, vkEnumeratePhysicalDevices),
        get_gpu_memory_properties:       get_inst_procaddr!(inst, vkGetPhysicalDeviceMemoryProperties),
        get_gpu_memory_properties2:      get_inst_procaddr!(inst, vkGetPhysicalDeviceMemoryProperties2),
        get_gpu_properties:              get_inst_procaddr!(inst, vkGetPhysicalDeviceProperties),
        get_gpu_features:                get_inst_procaddr!(inst, vkGetPhysicalDeviceFeatures),
        get_gpu_queue_family_properties: get_inst_procaddr!(inst, vkGetPhysicalDeviceQueueFamilyProperties),
        get_gpu_surface_present_modes:   get_inst_procaddr!(inst, vkGetPhysicalDeviceSurfacePresentModesKHR),
        get_gpu_surface_capabilities:    get_inst_procaddr!(inst, vkGetPhysicalDeviceSurfaceCapabilitiesKHR),
        get_gpu_surface_formats:         get_inst_procaddr!(inst, vkGetPhysicalDeviceSurfaceFormatsKHR),
        get_gpu_surface_support:         get_inst_procaddr!(inst, vkGetPhysicalDeviceSurfaceSupportKHR),
        get_gpu_format_properties:       get_inst_procaddr!(inst, vkGetPhysicalDeviceFormatProperties),
        #[cfg(target_os = "linux")]
        create_wayland_surface:     get_inst_procaddr_optional!(inst, vkCreateWaylandSurfaceKHR),
        #[cfg(target_os = "linux")]
        create_xlib_surface:        get_inst_procaddr_optional!(inst, vkCreateXlibSurfaceKHR),
        #[cfg(target_os = "windows")]
        create_win32_surface:       get_inst_procaddr_optional!(inst, vkCreateWin32SurfaceKHR),
        create_debug_messenger,
        destroy_debug_messenger,
    };

    Ok(funcs)
}

pub fn load_debug_messenger() {
    todo!()
}

pub fn load_device_functions(gbl: &GlobalFnTable, inst: VkInstance, device: VkDevice) -> Result<DeviceFnTable, String> {
    let get_device_procaddr: FN_vkGetDeviceProcAddr = get_vk_procaddr!(gbl.get_inst_procaddr, inst, vkGetDeviceProcAddr);

    macro_rules! get_device_procaddr {
        ($name:ident) => {
            get_vk_procaddr!(get_device_procaddr, device, $name)
        };
    }

    let funcs = DeviceFnTable {
        acquire_next_image:           get_device_procaddr!(vkAcquireNextImageKHR),
        alloc_command_buffers:        get_device_procaddr!(vkAllocateCommandBuffers),
        alloc_descriptor_sets:        get_device_procaddr!(vkAllocateDescriptorSets),
        alloc_memory:                 get_device_procaddr!(vkAllocateMemory),
        begin_command_buffer:         get_device_procaddr!(vkBeginCommandBuffer),
        bind_buffer_memory:           get_device_procaddr!(vkBindBufferMemory),
        cmd_begin_render_pass:        get_device_procaddr!(vkCmdBeginRenderPass),
        cmd_copy_buffer:              get_device_procaddr!(vkCmdCopyBuffer),
        cmd_end_render_pass:          get_device_procaddr!(vkCmdEndRenderPass),
        cmd_pipeline_barrier:         get_device_procaddr!(vkCmdPipelineBarrier),
        create_buffer:                get_device_procaddr!(vkCreateBuffer),
        create_command_pool:          get_device_procaddr!(vkCreateCommandPool),
        create_descriptor_pool:       get_device_procaddr!(vkCreateDescriptorPool),
        create_descriptor_set_layout: get_device_procaddr!(vkCreateDescriptorSetLayout),
        create_image:                 get_device_procaddr!(vkCreateImage),
        create_image_view:            get_device_procaddr!(vkCreateImageView),
        create_fence:                 get_device_procaddr!(vkCreateFence),
        create_framebuffer:           get_device_procaddr!(vkCreateFramebuffer),
        create_pipeline_cache:        get_device_procaddr!(vkCreatePipelineCache),
        create_pipeline_layout:       get_device_procaddr!(vkCreatePipelineLayout),
        create_render_pass:           get_device_procaddr!(vkCreateRenderPass),
        create_sampler:               get_device_procaddr!(vkCreateSampler),
        create_semaphore:             get_device_procaddr!(vkCreateSemaphore),
        create_shader_module:         get_device_procaddr!(vkCreateShaderModule),
        create_swapchain:             get_device_procaddr!(vkCreateSwapchainKHR),
        destroy_buffer:               get_device_procaddr!(vkDestroyBuffer),
        destroy_command_pool:         get_device_procaddr!(vkDestroyCommandPool),
        destroy_device:               get_device_procaddr!(vkDestroyDevice),
        destroy_fence:                get_device_procaddr!(vkDestroyFence),
        destroy_framebuffer:          get_device_procaddr!(vkDestroyFramebuffer),
        destroy_image:                get_device_procaddr!(vkDestroyImage),
        destroy_image_view:           get_device_procaddr!(vkDestroyImageView),
        destroy_render_pass:          get_device_procaddr!(vkDestroyRenderPass),
        destroy_semaphore:            get_device_procaddr!(vkDestroySemaphore),
        destroy_swapchain:            get_device_procaddr!(vkDestroySwapchainKHR),
        wait_idle:                    get_device_procaddr!(vkDeviceWaitIdle),
        end_command_buffer:           get_device_procaddr!(vkEndCommandBuffer),
        flush_mapped_memory_ranges:   get_device_procaddr!(vkFlushMappedMemoryRanges),
        free_command_buffers:         get_device_procaddr!(vkFreeCommandBuffers),
        free_memory:                  get_device_procaddr!(vkFreeMemory),
        get_buffer_memory_reqs:       get_device_procaddr!(vkGetBufferMemoryRequirements),
        get_queue:                    get_device_procaddr!(vkGetDeviceQueue),
        get_swapchain_images:         get_device_procaddr!(vkGetSwapchainImagesKHR),
        map_memory:                   get_device_procaddr!(vkMapMemory),
        queue_present:                get_device_procaddr!(vkQueuePresentKHR),
        queue_submit:                 get_device_procaddr!(vkQueueSubmit),
        queue_submit2:                get_device_procaddr!(vkQueueSubmit2),
        reset_command_buffer:         get_device_procaddr!(vkResetCommandBuffer),
        reset_command_pool:           get_device_procaddr!(vkResetCommandPool),
        reset_fences:                 get_device_procaddr!(vkResetFences),
        unmap_memory:                 get_device_procaddr!(vkUnmapMemory),
        update_descriptor_sets:       get_device_procaddr!(vkUpdateDescriptorSets),
        wait_for_fences:              get_device_procaddr!(vkWaitForFences),

        invalidate_mapped_memory_ranges: get_device_procaddr!(vkInvalidateMappedMemoryRanges),
        bind_image_memory:               get_device_procaddr!(vkBindImageMemory),
        get_image_memory_reqs:           get_device_procaddr!(vkGetImageMemoryRequirements),
        get_buffer_memory_reqs2:         get_device_procaddr!(vkGetBufferMemoryRequirements2),
        get_image_memory_reqs2:          get_device_procaddr!(vkGetImageMemoryRequirements2),
        bind_buffer_memory2:             get_device_procaddr!(vkBindBufferMemory2),
        bind_image_memory2:              get_device_procaddr!(vkBindImageMemory2),
        get_device_buffer_memory_reqs:   get_device_procaddr!(vkGetDeviceBufferMemoryRequirements),
        get_device_image_memory_reqs:    get_device_procaddr!(vkGetDeviceImageMemoryRequirements),

        cmd_pipeline_barrier2:           get_device_procaddr!(vkCmdPipelineBarrier2),
        cmd_clear_color_image:           get_device_procaddr!(vkCmdClearColorImage),
        cmd_blit_image2:                 get_device_procaddr!(vkCmdBlitImage2),

        reset_descriptor_pool:           get_device_procaddr!(vkResetDescriptorPool),
        destroy_descriptor_pool:         get_device_procaddr!(vkDestroyDescriptorPool),
        destroy_descriptor_set_layout:   get_device_procaddr!(vkDestroyDescriptorSetLayout),
        destroy_shader_module:           get_device_procaddr!(vkDestroyShaderModule),
        destroy_pipeline_layout:         get_device_procaddr!(vkDestroyPipelineLayout),
        create_compute_pipeline:         get_device_procaddr!(vkCreateComputePipelines),
        create_graphics_pipeline:        get_device_procaddr!(vkCreateGraphicsPipelines),
        destroy_pipeline:                get_device_procaddr!(vkDestroyPipeline),
        cmd_bind_pipeline:               get_device_procaddr!(vkCmdBindPipeline),
        cmd_bind_descriptor_sets:        get_device_procaddr!(vkCmdBindDescriptorSets),
        cmd_dispatch:                    get_device_procaddr!(vkCmdDispatch),
        cmd_begin_rendering:             get_device_procaddr!(vkCmdBeginRendering),
        cmd_end_rendering:               get_device_procaddr!(vkCmdEndRendering),
        cmd_set_scissor:                 get_device_procaddr!(vkCmdSetScissor),
        cmd_set_viewport:                get_device_procaddr!(vkCmdSetViewport),
        cmd_draw:                        get_device_procaddr!(vkCmdDraw),
        cmd_push_constants:              get_device_procaddr!(vkCmdPushConstants),
        get_device_address:              get_device_procaddr!(vkGetBufferDeviceAddress),
        cmd_bind_index_buffer:           get_device_procaddr!(vkCmdBindIndexBuffer),
        cmd_draw_indexed:                get_device_procaddr!(vkCmdDrawIndexed),
        cmd_copy_buffer_to_image:        get_device_procaddr!(vkCmdCopyBufferToImage),
        destroy_sampler:                 get_device_procaddr!(vkDestroySampler),
    };

    Ok(funcs)
}

/* ======================================================================== */
/* Helper Types                                                             */

pub enum QueueType {
    Present,
    Graphics,
    Compute,
    Transfer,
}

/* ======================================================================== */
/* Helper Functions                                                         */

#[inline(always)]
pub fn make_command_buffer_begin_info(usage_flags: VkCommandBufferUsageFlags) -> VkCommandBufferBeginInfo {
    return VkCommandBufferBeginInfo{
        sType:            VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
        pNext:            ptr::null(),
        flags:            usage_flags,
        pInheritanceInfo: ptr::null(),
    };
}

#[inline(always)]
pub fn make_image_subresource_range(aspect_mask: VkImageAspectFlags) -> VkImageSubresourceRange {
    return VkImageSubresourceRange{
        aspectMask:     aspect_mask,
        baseMipLevel:   0,
        levelCount:     VK_REMAINING_MIP_LEVELS as u32,
        baseArrayLayer: 0,
        layerCount:     VK_REMAINING_ARRAY_LAYERS as u32,
    };
}

#[inline(always)]
pub fn make_semaphore_submit_info(stage_mask: VkPipelineStageFlags2, semaphore: VkSemaphore) -> VkSemaphoreSubmitInfo {
    return VkSemaphoreSubmitInfo{
        sType:       VK_STRUCTURE_TYPE_SEMAPHORE_SUBMIT_INFO,
        pNext:       ptr::null(),
        semaphore,
        value:       1,
        stageMask:   stage_mask,
        deviceIndex: 0,
    };
}

#[inline(always)]
pub fn make_command_buffer_submit_info(cmd_buffer: VkCommandBuffer) -> VkCommandBufferSubmitInfo
{
    return VkCommandBufferSubmitInfo{
        sType:         VK_STRUCTURE_TYPE_COMMAND_BUFFER_SUBMIT_INFO,
        pNext:         ptr::null(),
        commandBuffer: cmd_buffer,
        deviceMask:    0,
    };
}

#[inline(always)]
pub fn make_submit_info(
    cmd_buffer_submit_info: VkCommandBufferSubmitInfo,
    signal_semaphore_info:  Option<VkSemaphoreSubmitInfo>,
    wait_semaphore_info:    Option<VkSemaphoreSubmitInfo>) -> VkSubmitInfo2
{
    let p_wait_info = if let Some(wait) = wait_semaphore_info {
        &wait
    } else {
        ptr::null()
    };

    let p_signal_info = if let Some(signal) = signal_semaphore_info {
        &signal
    } else {
        ptr::null()
    };

    return VkSubmitInfo2{
        sType:                    VK_STRUCTURE_TYPE_SUBMIT_INFO_2,
        pNext:                    ptr::null(),
        flags:                    0,
        waitSemaphoreInfoCount:   if wait_semaphore_info.is_some() { 1 } else { 0 },
        pWaitSemaphoreInfos:      p_wait_info,
        commandBufferInfoCount:   1,
        pCommandBufferInfos:      &cmd_buffer_submit_info,
        signalSemaphoreInfoCount: if signal_semaphore_info.is_some() { 1 } else { 0 },
        pSignalSemaphoreInfos:    p_signal_info,
    };
}

#[inline(always)]
pub fn make_image_ci(format: VkFormat, usage_flags: VkImageUsageFlags, extent: VkExtent3D) -> VkImageCreateInfo
{
    VkImageCreateInfo{
        sType:                 VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
        pNext:                 ptr::null(),
        imageType:             VK_IMAGE_TYPE_2D,
        format,
        extent,
        mipLevels:             1, //TODO: this should be configurable
        arrayLayers:           1,
        samples:               VK_SAMPLE_COUNT_1_BIT,   //for MSAA. not using it by default, so default it to 1 sample per pixel.
        tiling:                VK_IMAGE_TILING_OPTIMAL, //optimal tiling, so image is stored on the best gpu format
        usage:                 usage_flags,
        flags:                 0,
        sharingMode:           VK_SHARING_MODE_EXCLUSIVE,
        queueFamilyIndexCount: 0,
        pQueueFamilyIndices:   ptr::null(),
        initialLayout:         VK_IMAGE_LAYOUT_UNDEFINED,
    }

    // Tiling
    //   - If we want to read the image data from cpu, we would need to use tiling LINEAR (simple 2D array)
}

#[inline(always)]
pub fn make_image_view_ci(format: VkFormat, image: VkImage, aspect_flags: VkImageAspectFlags) -> VkImageViewCreateInfo
{
    // build an image-view for the depth image to use for rendering
    return VkImageViewCreateInfo{
        sType:            VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
        pNext:            ptr::null(),
        flags:            0,
                          image,
        viewType:         VK_IMAGE_VIEW_TYPE_2D,
                          format,
        components:       VkComponentMapping::default(),
        subresourceRange: VkImageSubresourceRange{
            aspectMask:     aspect_flags,
            baseMipLevel:   0,
            levelCount:     1,
            baseArrayLayer: 0,
            layerCount:     1,
        },
    };
}

#[inline(always)]
pub fn make_color_attachment_info(view: VkImageView, clear: Option<VkClearValue>, layout: VkImageLayout) -> VkRenderingAttachmentInfo {
    let mut attachment_info = VkRenderingAttachmentInfo::default();
    attachment_info.imageView   = view;
    attachment_info.imageLayout = layout;
    attachment_info.loadOp      = if clear.is_some() { VK_ATTACHMENT_LOAD_OP_CLEAR } else { VK_ATTACHMENT_LOAD_OP_LOAD };
    attachment_info.storeOp     = VK_ATTACHMENT_STORE_OP_STORE;
    attachment_info.clearValue  = if let Some(c) = clear { c } else { Default::default() };

    return attachment_info;
}

#[inline(always)]
pub fn make_depth_attachment_info(view: VkImageView, layout: VkImageLayout) -> VkRenderingAttachmentInfo {
    let mut attachment_info = VkRenderingAttachmentInfo::default();
    attachment_info.imageView   = view;
    attachment_info.imageLayout = layout;
    attachment_info.loadOp      = VK_ATTACHMENT_LOAD_OP_CLEAR;
    attachment_info.storeOp     = VK_ATTACHMENT_STORE_OP_STORE;
    attachment_info.clearValue  = VkClearValue{
        depthStencil: VkClearDepthStencilValue{
            depth:   0.0,
            stencil: 0,
        },
    };

    return attachment_info;
}

#[inline(always)]
pub fn make_rendering_info(render_extent: VkExtent2D, color_attachment: *const VkRenderingAttachmentInfo, depth_attachment: *const VkRenderingAttachmentInfo) -> VkRenderingInfo {
    let mut render_info = VkRenderingInfo::default();
    render_info.renderArea           = VkRect2D{ offset: VkOffset2D::default(), extent: render_extent };
    render_info.layerCount           = 1;
    render_info.colorAttachmentCount = 1;
    render_info.pColorAttachments    = color_attachment;
    render_info.pDepthAttachment     = depth_attachment;;

    return render_info;
}

#[inline(always)]
pub fn make_push_constant_range(offset: u32, size: u32, stage_flags: VkShaderStageFlags) -> VkPushConstantRange {
    return VkPushConstantRange{
        stageFlags: stage_flags,
        offset,
        size,
    };
}

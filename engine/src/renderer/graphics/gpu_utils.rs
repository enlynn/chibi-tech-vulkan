use std::ptr;

use crate::core::os;
use crate::util::ffi::*;

use super::api::*;
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

    get_inst_procaddr:                       FN_vkGetInstanceProcAddr,
    create_instance:                         FN_vkCreateInstance,
    enumerate_instance_extension_properties: FN_vkEnumerateInstanceExtensionProperties,
    enumerate_instance_layer_properties:     FN_vkEnumerateInstanceLayerProperties,
}

impl GlobalFnTable {
    pub fn enumerate_instance_extensions(&self) -> Vec<String> {
        todo!()
    }

    pub fn enumerate_instance_layers(&self) -> Vec<String> {
        todo!()
    }
}

pub struct InstanceFnTable
{
    create_device:              FN_vkCreateDevice,
    destroy_debug_messenger:    PFN_vkDestroyDebugUtilsMessengerEXT,
    destroy_instance:           FN_vkDestroyInstance,
    destroy_surface:            FN_vkDestroySurfaceKHR,
    enum_device_ext_props:      FN_vkEnumerateDeviceExtensionProperties,
    enum_physical_devices:      FN_vkEnumeratePhysicalDevices,
    get_phy_mem_props:          FN_vkGetPhysicalDeviceMemoryProperties,
    get_phy_props:              FN_vkGetPhysicalDeviceProperties,
    get_phy_queue_family_props: FN_vkGetPhysicalDeviceQueueFamilyProperties,
    get_phy_surface_caps:       FN_vkGetPhysicalDeviceSurfaceCapabilitiesKHR,
    get_phy_surface_support:    FN_vkGetPhysicalDeviceSurfaceSupportKHR,
    #[cfg(target_os = "linux")]
    create_wayland_surface:     PFN_vkCreateWaylandSurfaceKHR,
    #[cfg(target_os = "linux")]
    create_xlib_surface:        PFN_vkCreateXlibSurfaceKHR,
    #[cfg(target_os = "windows")]
    create_win32_surface:       PFN_vkCreateWin32SurfaceKHR,
}

impl InstanceFnTable {
    pub fn enumerate_device_extensions(&self) {
        todo!()
    }

    pub fn enumerate_gpus(&self) {
        todo!()
    }
}

pub struct DeviceFnTable
{
    acquire_next_image:           FN_vkAcquireNextImageKHR,
    alloc_command_buffers:        FN_vkAllocateCommandBuffers,
    alloc_descriptor_sets:        FN_vkAllocateDescriptorSets,
    alloc_memory:                 FN_vkAllocateMemory,
    begin_command_buffer:         FN_vkBeginCommandBuffer,
    bind_buffer_memory:           FN_vkBindBufferMemory,
    cmd_begin_render_pass:        FN_vkCmdBeginRenderPass,
    cmd_copy_buffer:              FN_vkCmdCopyBuffer,
    cmd_end_render_pass:          FN_vkCmdEndRenderPass,
    cmd_pipeline_barrier:         FN_vkCmdPipelineBarrier,
    create_buffer:                FN_vkCreateBuffer,
    create_command_pool:          FN_vkCreateCommandPool,
    create_descriptor_pool:       FN_vkCreateDescriptorPool,
    create_descriptor_set_layout: FN_vkCreateDescriptorSetLayout,
    create_image:                 FN_vkCreateImage,
    create_image_view:            FN_vkCreateImageView,
    create_fence:                 FN_vkCreateFence,
    create_framebuffer:           FN_vkCreateFramebuffer,
    create_pipeline_cache:        FN_vkCreatePipelineCache,
    create_pipeline_layout:       FN_vkCreatePipelineLayout,
    create_render_pass:           FN_vkCreateRenderPass,
    create_sampler:               FN_vkCreateSampler,
    create_semaphore:             FN_vkCreateSemaphore,
    create_shader_module:         FN_vkCreateShaderModule,
    create_swapchain:             FN_vkCreateSwapchainKHR,
    destroy_buffer:               FN_vkDestroyBuffer,
    destroy_command_pool:         FN_vkDestroyCommandPool,
    destroy_device:               FN_vkDestroyDevice,
    destroy_fence:                FN_vkDestroyFence,
    destroy_framebuffer:          FN_vkDestroyFramebuffer,
    destroy_image:                FN_vkDestroyImage,
    destroy_image_view:           FN_vkDestroyImageView,
    destroy_semaphore:            FN_vkDestroySemaphore,
    destroy_render_pass:          FN_vkDestroyRenderPass,
    destroy_swapchain:            FN_vkDestroySwapchainKHR,
    wait_idle:                    FN_vkDeviceWaitIdle,
    end_command_buffer:           FN_vkEndCommandBuffer,
    flush_mapped_memory_ranges:   FN_vkFlushMappedMemoryRanges,
    free_command_buffers:         FN_vkFreeCommandBuffers,
    free_memory:                  FN_vkFreeMemory,
    get_buffer_memory_reqs:       FN_vkGetBufferMemoryRequirements,
    get_queue:                    FN_vkGetDeviceQueue,
    get_swapchain_images:         FN_vkGetSwapchainImagesKHR,
    map_memory:                   FN_vkMapMemory,
    queue_present:                FN_vkQueuePresentKHR,
    queue_submit:                 FN_vkQueueSubmit,
    reset_command_buffer:         FN_vkResetCommandBuffer,
    reset_command_pool:           FN_vkResetCommandPool,
    reset_fences:                 FN_vkResetFences,
    unmap_memory:                 FN_vkUnmapMemory,
    update_descriptor_sets:       FN_vkUpdateDescriptorSets,
    wait_for_fences:              FN_vkWaitForFences,
}

/* ======================================================================== */
/* Vulkan Helper Functions                                                  */

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

    let destroy_debug_messenger = if consts::ENABLE_DEBUG_MESSENGER {
        Some(get_inst_procaddr!(inst, vkDestroyDebugUtilsMessengerEXT))
    } else {
        None
    };

    let funcs = InstanceFnTable {
        create_device:              get_inst_procaddr!(inst, vkCreateDevice),
        destroy_debug_messenger:    destroy_debug_messenger,
        destroy_instance:           get_inst_procaddr!(inst, vkDestroyInstance),
        destroy_surface:            get_inst_procaddr!(inst, vkDestroySurfaceKHR),
        enum_device_ext_props:      get_inst_procaddr!(inst, vkEnumerateDeviceExtensionProperties),
        enum_physical_devices:      get_inst_procaddr!(inst, vkEnumeratePhysicalDevices),
        get_phy_mem_props:          get_inst_procaddr!(inst, vkGetPhysicalDeviceMemoryProperties),
        get_phy_props:              get_inst_procaddr!(inst, vkGetPhysicalDeviceProperties),
        get_phy_queue_family_props: get_inst_procaddr!(inst, vkGetPhysicalDeviceQueueFamilyProperties),
        get_phy_surface_caps:       get_inst_procaddr!(inst, vkGetPhysicalDeviceSurfaceCapabilitiesKHR),
        get_phy_surface_support:    get_inst_procaddr!(inst, vkGetPhysicalDeviceSurfaceSupportKHR),
        #[cfg(target_os = "linux")]
        create_wayland_surface:     get_inst_procaddr_optional!(inst, vkCreateWaylandSurfaceKHR),
        #[cfg(target_os = "linux")]
        create_xlib_surface:        get_inst_procaddr_optional!(inst, vkCreateXlibSurfaceKHR),
        #[cfg(target_os = "windows")]
        create_win32_surface:       get_inst_procaddr_optional!(inst, vkCreateWin32SurfaceKHR),
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
        reset_command_buffer:         get_device_procaddr!(vkResetCommandBuffer),
        reset_command_pool:           get_device_procaddr!(vkResetCommandPool),
        reset_fences:                 get_device_procaddr!(vkResetFences),
        unmap_memory:                 get_device_procaddr!(vkUnmapMemory),
        update_descriptor_sets:       get_device_procaddr!(vkUpdateDescriptorSets),
        wait_for_fences:              get_device_procaddr!(vkWaitForFences),
    };

    Ok(funcs)
}

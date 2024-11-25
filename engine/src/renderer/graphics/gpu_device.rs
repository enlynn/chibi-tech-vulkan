use crate::util::ffi::*;
use crate::window::NativeSurface;

use super::api;
use super::consts;
use super::gpu_device_context as context;
use super::gpu_swapchain::{Swapchain, SwapchainFnTable};
use super::gpu_utils as util;
use super::gpu_utils::call_throw;
use super::gpu_command_pool::{CommandPool, CommandPoolFnTable};
use super::gpu_command_buffer::{CommandBuffer, CommandBufferFnTable};

use std::borrow::Borrow;
use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::ptr;
use std::rc::Rc;

//
// TODO:
// - Device Extensions require modifying 2 locations (is device valid, and device creation), which is not great. centralize this into an array.
// - HDR Support (select_surface_format)
//

pub struct Features {
    pub prefer_hdr: bool,
}

impl Default for Features {
    fn default() -> Self {
        Self{
            prefer_hdr: false,
        }
    }
}

pub struct CreateInfo {
    pub features:         Features,
    pub surface:          NativeSurface,
    pub software_version: u32,
    pub software_name:    String,
}

pub struct Instance {
    // Vulkan-Loaded functions
    glb_fns:              util::GlobalFnTable,
    inst_fns:             util::InstanceFnTable,
    handle:               api::VkInstance,
    // meta information about the VkInstance
    requested_layers:     Vec<CString>,
    requested_extensions: Vec<CString>,
    software_name:        CString,
    engine_name:          CString,
}

type QueueFamily = u32;

#[derive(Default)]
pub struct GpuQueueFamilies {
    graphics: Option<QueueFamily>,
    present:  Option<QueueFamily>,
    compute:  Option<QueueFamily>,
    transfer: Option<QueueFamily>,
}

#[derive(Default)]
pub struct SwapchainSupportInfo {
    capabilities:  api::VkSurfaceCapabilitiesKHR,
    formats:       Vec<api::VkSurfaceFormatKHR>,
    present_modes: Vec<api::VkPresentModeKHR>,
}

struct GpuFnTable {
    pub get_gpu_format_properties: api::FN_vkGetPhysicalDeviceFormatProperties,
}

pub struct Gpu {
    fns:                                    GpuFnTable,
    pub handle:                             api::VkPhysicalDevice,
    pub properties:                         api::VkPhysicalDeviceProperties,
    pub features:                           api::VkPhysicalDeviceFeatures,
    pub memory_properties:                  api::VkPhysicalDeviceMemoryProperties,
    pub queue_infos:                        GpuQueueFamilies,
    pub swapchain_support_info:             SwapchainSupportInfo,
    pub supports_device_local_host_visible: bool,
}

pub struct Display {
    //todo:
}

pub struct Surface {
    handle: api::VkSurfaceKHR,
}

pub struct Device {
    pub fns:      util::DeviceFnTable,
    pub handle:   api::VkDevice,
    allocator: api::VmaAllocator,

    instance: Instance,
    surface:  Surface,
    gpus:     Vec<Rc<Gpu>>,
    //displays: Vec<Rc<Display>>,
    gpu:      Rc<Gpu>,
    //display: Rc<Display>,
}

unsafe extern "C" fn debug_callback(
    severity: api::VkDebugUtilsMessageSeverityFlagBitsEXT,
    _message_type: api::VkDebugUtilsMessageTypeFlagsEXT,
    data: *const api::VkDebugUtilsMessengerCallbackDataEXT,
    _: *mut std::os::raw::c_void,
) -> api::VkBool32 {
    if (severity & api::VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT) != 0 {
        println!("[VERBOSE]: {:?}", CStr::from_ptr((*data).pMessage));
    } else if (severity & api::VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT) != 0 {
        println!("[INFO]: {:?}", CStr::from_ptr((*data).pMessage));
    } else if (severity & api::VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT) != 0 {
        println!("[WARNING]: {:?}", CStr::from_ptr((*data).pMessage));
    } else if (severity & api::VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT) != 0 {
        println!("[ERROR]: {:?}", CStr::from_ptr((*data).pMessage));
    }

    api::VK_FALSE
}

impl Instance {
    pub fn new(software_version: u32, software_name: &str) -> Result<Instance, String> {
        // load vulkan functions
        //
        let global_fns: util::GlobalFnTable = match util::load_vulkan_proc_addr() {
            Ok(fns) => fns,
            Err(reason) => panic!("Failed to load vulkan library: {}", reason),
        };

        // Build list of validation layers, if enabled and available
        //
        let desired_validation_layer =
            byte_array_as_cstr!(consts::VK_LAYER_KHRONOS_VALIDATION_LAYER_NAME);

        let mut instance_layer_strings = Vec::<CString>::new();
        let mut instance_layers        = Vec::<*const std::os::raw::c_char>::new();

        if consts::ENABLE_DEBUG_LAYER {
            let validation_layers = global_fns.enumerate_instance_layers();

            for layer in validation_layers.iter() {
                if char_array_as_cstr!(layer.layerName) == desired_validation_layer {
                    let string: CString = desired_validation_layer.into();
                    instance_layers.push(string.as_ptr());
                    instance_layer_strings.push(string);

                    break;
                }
            }

            if instance_layer_strings.len() == 0 {
                println!("[WARN] :: Instance::new :: Requested validation layers, but they were not found.");
            }
        }

        // Populate the debug messenger info, we'll load the function once VkInstance has been created.
        //

        let severities = api::VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT
            | api::VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT;

        let message_types = api::VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT
            | api::VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT
            | api::VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT;

        let mut debug_messenger_ci = api::VkDebugUtilsMessengerCreateInfoEXT::default();
        debug_messenger_ci.messageSeverity = severities;
        debug_messenger_ci.messageType     = message_types;
        debug_messenger_ci.pfnUserCallback = Some(debug_callback);

        let p_next: *const std::os::raw::c_void = if consts::ENABLE_DEBUG_LAYER {
            &debug_messenger_ci as *const _ as *const std::os::raw::c_void
        } else {
            std::ptr::null()
        };

        // Build list of instance extensions
        //

        let mut instance_ext_strings = Vec::<CString>::new();
        let mut instance_exts        = Vec::<*const std::os::raw::c_char>::new();

        let available_extensions = global_fns.enumerate_instance_extensions();

        // We want 2 specific platform extensions:
        // 1. Surface KHR extension
        // 2. Platform Surface KHR extension
        // 3. Physical Device Properties 2
        let mut surface_ext_found          = false;
        let mut platform_surface_ext_found = false;
        let mut device_props2_ext_found    = false;

        // TODO(enlynn): other operating systems
        let platform_surface_ext = if cfg!(target_os = "linux") {
            if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
                match session_type.as_str() {
                    "x11"     => byte_array_as_cstr!(api::VK_KHR_XLIB_SURFACE_EXTENSION_NAME),
                    "wayland" => byte_array_as_cstr!(api::VK_KHR_WAYLAND_SURFACE_EXTENSION_NAME),
                    _         => return Err("Unsupported window manager".to_string()),
                }
            } else {
                return Err("Unsupported window manager".to_string());
            }
        } else {
            return Err("Unsupported operating system".to_string());
        };

        // Verify the extensions are available
        for extension in available_extensions.iter() {
            let ext_c_str = char_array_as_cstr!(extension.extensionName);

            if ext_c_str == byte_array_as_cstr!(api::VK_KHR_SURFACE_EXTENSION_NAME) {
                let string: CString = ext_c_str.into();

                instance_exts.push(string.as_ptr());
                instance_ext_strings.push(string);

                surface_ext_found = true;
            } else if ext_c_str == byte_array_as_cstr!(api::VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME) {
                let string: CString = ext_c_str.into();

                instance_exts.push(string.as_ptr());
                instance_ext_strings.push(string);

                device_props2_ext_found = true;
            } else if ext_c_str == platform_surface_ext {
                let string: CString = ext_c_str.into();

                instance_exts.push(string.as_ptr());
                instance_ext_strings.push(string);

                platform_surface_ext_found = true;
            } else if consts::ENABLE_DEBUG_LAYER {
                if ext_c_str == byte_array_as_cstr!(api::VK_EXT_DEBUG_UTILS_EXTENSION_NAME) {
                    let string: CString = ext_c_str.into();

                    instance_exts.push(string.as_ptr());
                    instance_ext_strings.push(string);
                }
            }
        }

        // These extensions are required for rendering to the Swapchain, so failing to find them is a fatal error.
        if !surface_ext_found {
            return Err("Surface extension for Vulkan not found".to_string());
        }

        if !device_props2_ext_found {
            return Err("Failed to find instance extension: VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME".to_string());
        }

        if !platform_surface_ext_found {
            return Err("Platform surface extension for Vulkan not found".to_string());
        }

        // Create the VkInstance
        //
        let software_name = CString::new(software_name).unwrap();
        let engine_name   = CString::new("ChibiTech").unwrap();

        let mut app_info = api::VkApplicationInfo::default();
        app_info.pEngineName        = engine_name.as_ptr();
        app_info.engineVersion      = crate::ENGINE_VERSION;
        app_info.pApplicationName   = software_name.as_ptr();
        app_info.applicationVersion = software_version;
        //api version is 1.3 by default

        let mut instance_ci = api::VkInstanceCreateInfo::default();
        instance_ci.pNext                   = p_next;
        instance_ci.pApplicationInfo        = &app_info as *const _;
        instance_ci.enabledLayerCount       = instance_layers.len() as u32;
        instance_ci.ppEnabledLayerNames     = instance_layers.as_ptr();
        instance_ci.enabledExtensionCount   = instance_exts.len() as u32;
        instance_ci.ppEnabledExtensionNames = instance_exts.as_ptr();

        let mut instance: api::VkInstance = std::ptr::null_mut();
        util::call_throw!(
            global_fns.create_instance,
            &instance_ci as *const _,
            std::ptr::null(),
            &mut instance as *mut _
        );

        // Load Instance-level Functions
        //
        let instance_fns: util::InstanceFnTable =
            match util::load_instance_functions(&global_fns, instance) {
                Ok(fns) => fns,
                Err(reason) => panic!("Failed to load vulkan library: {}", reason),
            };

        // Create the debug messenger
        //

        let debug_messenger = if consts::ENABLE_DEBUG_LAYER {
            let mut ptr: api::VkDebugUtilsMessengerEXT = std::ptr::null_mut();

            let mut result: i32 = api::VK_SUCCESS;
            if let Some(create_debug_messenger) = instance_fns.create_debug_messenger {
                result = call!(
                    create_debug_messenger,
                    instance,
                    &debug_messenger_ci as *const _,
                    std::ptr::null_mut(),
                    &mut ptr as *mut _
                );
            } else {
                println!("[WARN] :: Instance::new :: Requested debug messenger, but failed to load the function.");
            }

            if result < 0 || ptr.is_null() {
                None
            } else {
                Some(ptr)
            }
        } else {
            None
        };

        Ok(Instance {
            glb_fns: global_fns,
            inst_fns: instance_fns,
            handle: instance,
            requested_layers: instance_layer_strings,
            requested_extensions: instance_ext_strings,
            software_name,
            engine_name,
        })
    }
}

impl Surface {
    pub fn new(instance: &Instance, native_surface: NativeSurface) -> Result<Surface, String> {
        let result = if cfg!(target_os = "linux") {
            if let NativeSurface::Wayland(native) = native_surface {
                let info = api::VkWaylandSurfaceCreateInfoKHR {
                    sType: api::VK_STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR,
                    pNext: std::ptr::null(),
                    flags: 0,
                    display: native.display as *mut api::wl_display,
                    surface: native.surface as *mut api::wl_surface,
                };

                let mut surf: MaybeUninit<_> = MaybeUninit::<api::VkSurfaceKHR>::uninit();
                util::call_throw!(
                    instance.inst_fns.create_wayland_surface.unwrap(),
                    instance.handle,
                    &info as *const _,
                    std::ptr::null(),
                    surf.as_mut_ptr()
                );
                Ok(Surface {
                    handle: unsafe { surf.assume_init() },
                })
            } else if let NativeSurface::X11(native) = native_surface {
                let info = api::VkXlibSurfaceCreateInfoKHR {
                    sType: api::VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR,
                    pNext: std::ptr::null(),
                    flags: 0,
                    dpy: native.display as *mut api::Display,
                    window: native.window as api::Window,
                };

                let mut surf = MaybeUninit::<api::VkSurfaceKHR>::uninit();
                util::call_throw!(
                    instance.inst_fns.create_xlib_surface.unwrap(),
                    instance.handle,
                    &info as *const _,
                    std::ptr::null(),
                    surf.as_mut_ptr()
                );
                Ok(Surface {
                    handle: unsafe { surf.assume_init() },
                })
            } else {
                panic!("Invalid native surface for linux.");
            }
        } else {
            Err("unsupported operating system".to_string())
        };

        result
    }
}

impl Gpu {
    fn get_queue_families(
        instance: &Instance,
        surface: &Surface,
        gpu: api::VkPhysicalDevice,
    ) -> GpuQueueFamilies {
        let mut result = GpuQueueFamilies::default();

        let queue_properties = instance.inst_fns.enumerate_gpu_queue_family_properties(gpu);

        // Iterate  over each queue family and select each queue of based on a score to determine if the queue
        // is a *unique* queue. If no unique queue is found, a duplicate is selected.
        let min_transfer_score: u32 = 255;
        let mut queue_family_index: u32 = 0;
        for property in queue_properties {
            let mut current_transfer_score: u8 = 0;

            // Graphics queue?
            if (property.queueFlags & api::VK_QUEUE_GRAPHICS_BIT) != 0 {
                result.graphics = Some(queue_family_index);
                current_transfer_score += 1;
            }

            // Compute queue?
            if (property.queueFlags & api::VK_QUEUE_COMPUTE_BIT) != 0 {
                result.compute = Some(queue_family_index);
                current_transfer_score += 1;
            }

            // Does this queue family support the present queue? If so, yoink it.
            let mut supports_present: api::VkBool32 = api::VK_FALSE;
            util::call_throw!(
                instance.inst_fns.get_gpu_surface_support,
                gpu,
                queue_family_index,
                surface.handle,
                &mut supports_present
            );

            if supports_present == api::VK_TRUE {
                result.present = Some(queue_family_index);
            }

            queue_family_index += 1;
        }

        return result;
    }

    pub fn query_swapchain_capabilities(
        instance: &Instance,
        surface:  &Surface,
        gpu:       api::VkPhysicalDevice,
    ) -> SwapchainSupportInfo {
        // Surface capabilities
        let mut capabilities_unsafe = MaybeUninit::<api::VkSurfaceCapabilitiesKHR>::uninit();
        util::call_throw!(
            instance.inst_fns.get_gpu_surface_capabilities,
            gpu,
            surface.handle,
            capabilities_unsafe.as_mut_ptr()
        );

        // Surface formats
        let formats = instance
            .inst_fns
            .enumerate_gpu_surface_formats(gpu, surface.handle);

        // Present modes
        let present_modes = instance
            .inst_fns
            .enumerate_gpu_present_modes(gpu, surface.handle);

        return SwapchainSupportInfo {
            capabilities: unsafe { capabilities_unsafe.assume_init() },
            formats,
            present_modes,
        };
    }

    fn does_gpu_meet_requirements(
        instance:     &Instance,
        surface:      &Surface,
        gpu:           api::VkPhysicalDevice,
        gpu_features: &api::VkPhysicalDeviceFeatures,
    ) -> bool {
        let queue_families = Self::get_queue_families(instance, surface, gpu);

        let has_present = queue_families.present.is_some();
        let has_graphics = queue_families.graphics.is_some();
        let has_transfer = queue_families.transfer.is_some();
        let has_compute = queue_families.compute.is_some();
        if !has_present || !has_graphics {
            println!("Missing required queues");
            return false;
        }

        let swapchain_info = Self::query_swapchain_capabilities(instance, surface, gpu);
        if swapchain_info.formats.is_empty() || swapchain_info.present_modes.is_empty() {
            // missing presentable surface
            println!("Missing presentation surface");
            return false;
        }

        // Check for sampler anisotropy
        const REQUIRE_ANISOTROPY: bool = true;
        if REQUIRE_ANISOTROPY && gpu_features.samplerAnisotropy != api::VK_TRUE {
            println!("Requested anisotropy, but not found.");
            return false;
        }

        // Make sure the gpu supports all required extensions
        let mut swapchain_extension_found = false;
        let mut semaphore_timelines_found = false;

        let extensions = instance.inst_fns.enumerate_device_extensions(gpu);
        for extension in extensions {
            let ext_c_str = char_array_as_cstr!(extension.extensionName);

            if ext_c_str == byte_array_as_cstr!(api::VK_KHR_SWAPCHAIN_EXTENSION_NAME) {
                swapchain_extension_found = true;
            } else if ext_c_str
                == byte_array_as_cstr!(api::VK_KHR_TIMELINE_SEMAPHORE_EXTENSION_NAME)
            {
                semaphore_timelines_found = true;
            }
        }

        if !swapchain_extension_found || !semaphore_timelines_found {
            // could not find required extensions
            println!("Could not find required extensions");
            return false;
        }

        true
    }

    fn enumerate_gpus(instance: &Instance, surface: &Surface) -> Vec<Rc<Gpu>> {
        let mut result = Vec::<Rc<Gpu>>::new();

        let vk_gpus = instance.inst_fns.enumerate_gpus(instance.handle);
        for gpu in vk_gpus.into_iter() {
            let mut properties_unsafe = MaybeUninit::<api::VkPhysicalDeviceProperties>::uninit();
            call!(
                instance.inst_fns.get_gpu_properties,
                gpu,
                properties_unsafe.as_mut_ptr()
            );

            let mut features_unsafe = MaybeUninit::<api::VkPhysicalDeviceFeatures>::uninit();
            call!(
                instance.inst_fns.get_gpu_features,
                gpu,
                features_unsafe.as_mut_ptr()
            );

            let mut memory_unsafe = MaybeUninit::<api::VkPhysicalDeviceMemoryProperties>::uninit();
            call!(
                instance.inst_fns.get_gpu_memory_properties,
                gpu,
                memory_unsafe.as_mut_ptr()
            );

            // let's unwrap the types
            let properties = unsafe { properties_unsafe.assume_init() };
            let features = unsafe { features_unsafe.assume_init() };
            let memory = unsafe { memory_unsafe.assume_init() };

            // Check if device supports local/host visible combo
            let mut supports_device_local_host_visible = false;
            for i in 0..memory.memoryTypeCount {
                let has_host_visible = (memory.memoryTypes[i as usize].propertyFlags
                    & api::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT)
                    != 0;
                let has_device_local = (memory.memoryTypes[i as usize].propertyFlags
                    & api::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT)
                    != 0;
                if has_host_visible && has_device_local {
                    supports_device_local_host_visible = true;
                    break;
                }
            }

            if !Gpu::does_gpu_meet_requirements(instance, surface, gpu, &features) {
                continue;
            }

            let gpu_fns = GpuFnTable{
                get_gpu_format_properties: instance.inst_fns.get_gpu_format_properties,
            };

            let adapter = Rc::new(Gpu {
                fns: gpu_fns,
                handle: gpu,
                properties,
                features,
                memory_properties: memory,
                queue_infos: Gpu::get_queue_families(instance, surface, gpu),
                swapchain_support_info: Gpu::query_swapchain_capabilities(instance, surface, gpu),
                supports_device_local_host_visible,
            });

            result.push(adapter);
        }

        result
    }

    pub fn require_portability_subset(&self, instance: &Instance) -> bool {
        let extensions = instance.inst_fns.enumerate_device_extensions(self.handle);
        for extension in extensions {
            let ext_c_str = char_array_as_cstr!(extension.extensionName);
            if ext_c_str == byte_array_as_cstr!(consts::VK_KHR_PORTABILITY_SUBSET_EXTENSION_NAME) {
                return true;
            }
        }
        return false;
    }

    pub fn select_surface_format(&self, prefer_hdr: bool) -> api::VkSurfaceFormatKHR {
        let has_format = |formats: &Vec<api::VkSurfaceFormatKHR>, desired_format: api::VkSurfaceFormatKHR| -> bool {
            for surface_format in formats {
                if (surface_format.format == desired_format.format && surface_format.colorSpace == desired_format.colorSpace) {
                    return true;
                }
            }

            return false;
        };

        const SDR_FORMAT: api::VkSurfaceFormatKHR = api::VkSurfaceFormatKHR{ format: api::VK_FORMAT_B8G8R8A8_UNORM, colorSpace: api::VK_COLOR_SPACE_SRGB_NONLINEAR_KHR };
        const HDR_FORMAT: api::VkSurfaceFormatKHR = api::VkSurfaceFormatKHR{ format: api::VK_FORMAT_UNDEFINED,      colorSpace: api::VK_COLOR_SPACE_MAX_ENUM_KHR       };

        if prefer_hdr && has_format(&self.swapchain_support_info.formats, HDR_FORMAT) {
            return HDR_FORMAT;
        }

        if has_format(&self.swapchain_support_info.formats, SDR_FORMAT) {
            return SDR_FORMAT;
        }

        println!("Failed to find available format. Choosing fist available format.");
        assert!(self.swapchain_support_info.formats.len() > 0);

        return self.swapchain_support_info.formats[0];
    }

    pub fn select_depth_format(&self) -> api::VkFormat {
        let mut depth_format = api::VK_FORMAT_UNDEFINED;

        const DEPTH_CANDIDATES: [api::VkFormat; 3] = [
            api::VK_FORMAT_D32_SFLOAT,
            api::VK_FORMAT_D32_SFLOAT_S8_UINT,
            api::VK_FORMAT_D24_UNORM_S8_UINT
        ];

        const DEPTH_FLAGS: u32 = api::VK_FORMAT_FEATURE_DEPTH_STENCIL_ATTACHMENT_BIT;

        for candidate in DEPTH_CANDIDATES {
            let mut format_props_unsafe: MaybeUninit<_> = MaybeUninit::<api::VkFormatProperties>::uninit();
            call!(self.fns.get_gpu_format_properties, self.handle, candidate, format_props_unsafe.as_mut_ptr());

            let format_props = unsafe { format_props_unsafe.assume_init() };

            let has_linear_tiling  = (format_props.linearTilingFeatures  & DEPTH_FLAGS) != 0;
            let has_optimal_tiling = (format_props.optimalTilingFeatures & DEPTH_FLAGS) != 0;
            if has_linear_tiling || has_optimal_tiling {
                depth_format = candidate;
                break;
            }
        }

        return depth_format;
    }
}

impl Device {
    pub fn new(create_info: CreateInfo) -> Device {
        let instance = match Instance::new(
            create_info.software_version,
            create_info.software_name.as_str(),
        ) {
            Ok(inst) => inst,
            Err(reason) => panic!("Failed to create vulkan instance: {}", reason),
        };

        let surface = match Surface::new(&instance, create_info.surface) {
            Ok(surf) => surf,
            Err(reason) => panic!("Failed to create vulkan surface: {}", reason),
        };

        let gpus = Gpu::enumerate_gpus(&instance, &surface);
        assert!(gpus.len() > 0);

        let chosen_gpu: Rc<Gpu> = Self::select_gpu(&gpus, None);

        //---------------------------------------------------------------------------------------//
        // Create Logical Device

        // 1. Create a list of unique queues
        const MAX_QUEUES: usize = 4;
        let all_queues: [Option<u32>; MAX_QUEUES] = [
            chosen_gpu.queue_infos.present,
            chosen_gpu.queue_infos.graphics,
            chosen_gpu.queue_infos.compute,
            chosen_gpu.queue_infos.transfer,
        ];

        let mut unique_queues: [u32; MAX_QUEUES] = [0; MAX_QUEUES];
        let mut queue_count = 0u32;

        for queue_option in all_queues {
            if let Some(queue) = queue_option {
                let mut found = false;
                for i in 0..queue_count {
                    if queue == unique_queues[i as usize] {
                        found = true;
                        break;
                    }
                }

                if !found {
                    unique_queues[queue_count as usize] = queue;
                    queue_count += 1;
                }
            }
        }

        let queue_priority: f32 = 1.0;

        let mut queue_ci: [api::VkDeviceQueueCreateInfo; MAX_QUEUES] = [api::VkDeviceQueueCreateInfo::default(); MAX_QUEUES];
        for i in 0..queue_count {
            queue_ci[i as usize].queueFamilyIndex = unique_queues[i as usize];
            queue_ci[i as usize].queueCount = 1;
            queue_ci[i as usize].pQueuePriorities = &queue_priority;
        }

        // 2. Enable Optional Features
        //   - Device Address (allows for direct GPU access)
        //   - Synchronization2
        //   - Timeline Semaphores
        use core::ffi::c_void;

        let mut feature_device_addr = api::VkPhysicalDeviceBufferDeviceAddressFeatures::default();
        feature_device_addr.bufferDeviceAddress = api::VK_TRUE;

        let feature_device_addr_ptr: *mut api::VkPhysicalDeviceBufferDeviceAddressFeatures = &mut feature_device_addr;

        let mut feature_sync2 = api::VkPhysicalDeviceSynchronization2Features::default();
        feature_sync2.synchronization2 = api::VK_TRUE;
        feature_sync2.pNext = feature_device_addr_ptr as *mut c_void;

        let feature_sync2_ptr: *mut api::VkPhysicalDeviceSynchronization2Features =
            &mut feature_sync2;

        let mut feature_timeline = api::VkPhysicalDeviceTimelineSemaphoreFeatures::default();
        feature_timeline.timelineSemaphore = api::VK_TRUE;
        feature_timeline.pNext = feature_sync2_ptr as *mut c_void;

        let feature_timeline_ptr: *mut api::VkPhysicalDeviceTimelineSemaphoreFeatures =
            &mut feature_timeline;

        //let enabled_features = api::VkPhysicalDeviceFeatures::default();
        // left here in case I want to override the defaults in the future

        let mut enabled_features2 = api::VkPhysicalDeviceFeatures2 {
            pNext: feature_timeline_ptr as *mut c_void,
            ..Default::default()
        };

        let enabled_features2_ptr: *mut api::VkPhysicalDeviceFeatures2 = &mut enabled_features2;

        // 3. Build the list of device extensions

        let mut extension_list_strings = Vec::<CString>::with_capacity(3);
        let mut extension_list = Vec::<*const std::os::raw::c_char>::with_capacity(3);

        let swapchain_ext_string: CString =
            byte_array_as_cstr!(api::VK_KHR_SWAPCHAIN_EXTENSION_NAME).into();
        let semaphore_ext_string: CString =
            byte_array_as_cstr!(api::VK_KHR_TIMELINE_SEMAPHORE_EXTENSION_NAME).into();

        extension_list.push(swapchain_ext_string.as_ptr());
        extension_list.push(semaphore_ext_string.as_ptr());

        extension_list_strings.push(swapchain_ext_string);
        extension_list_strings.push(semaphore_ext_string);

        if chosen_gpu.require_portability_subset(&instance) {
            let portability_ext_string: CString =
                byte_array_as_cstr!(consts::VK_KHR_PORTABILITY_SUBSET_EXTENSION_NAME).into();
            extension_list.push(portability_ext_string.as_ptr());
            extension_list_strings.push(portability_ext_string);
        }

        // 4. Create the Device!

        let mut device_ci = api::VkDeviceCreateInfo::default();
        device_ci.queueCreateInfoCount = queue_count;
        device_ci.pQueueCreateInfos = queue_ci.as_ptr();
        device_ci.enabledExtensionCount = extension_list.len() as u32;
        device_ci.ppEnabledExtensionNames = extension_list.as_mut_ptr();
        device_ci.pNext = enabled_features2_ptr as *mut c_void;

        let mut device_handle: api::VkDevice = std::ptr::null_mut();
        call_throw!(
            instance.inst_fns.create_device,
            chosen_gpu.handle,
            &device_ci as *const _,
            ptr::null(),
            &mut device_handle as *mut _
        );

        //---------------------------------------------------------------------------------------//
        // Load Device Functions

        let device_fns =
            util::load_device_functions(&instance.glb_fns, instance.handle, device_handle)
                .expect("Failed to load Vulkan Device level functions");

        //---------------------------------------------------------------------------------------//
        // Load the Vulkan Memory Allocator

        let vma_flags: api::VmaAllocatorCreateFlags =
            api::VMA_ALLOCATOR_CREATE_EXTERNALLY_SYNCHRONIZED_BIT | // app is single threaded at the moment.
            api::VMA_ALLOCATOR_CREATE_BUFFER_DEVICE_ADDRESS_BIT;

        let vma_fns = api::VmaVulkanFunctions{
            vkGetInstanceProcAddr:                   Some(instance.glb_fns.get_inst_procaddr),
            vkGetDeviceProcAddr:                     Some(instance.inst_fns.get_device_procaddr),
            vkGetPhysicalDeviceProperties:           Some(instance.inst_fns.get_gpu_properties),
            vkGetPhysicalDeviceMemoryProperties:     Some(instance.inst_fns.get_gpu_memory_properties),
            vkAllocateMemory:                        Some(device_fns.alloc_memory),
            vkFreeMemory:                            Some(device_fns.free_memory),
            vkMapMemory:                             Some(device_fns.map_memory),
            vkUnmapMemory:                           Some(device_fns.unmap_memory),
            vkFlushMappedMemoryRanges:               Some(device_fns.flush_mapped_memory_ranges),
            vkInvalidateMappedMemoryRanges:          Some(device_fns.invalidate_mapped_memory_ranges),
            vkBindBufferMemory:                      Some(device_fns.bind_buffer_memory),
            vkBindImageMemory:                       Some(device_fns.bind_image_memory),
            vkGetBufferMemoryRequirements:           Some(device_fns.get_buffer_memory_reqs),
            vkGetImageMemoryRequirements:            Some(device_fns.get_image_memory_reqs),
            vkCreateBuffer:                          Some(device_fns.create_buffer),
            vkDestroyBuffer:                         Some(device_fns.destroy_buffer),
            vkCreateImage:                           Some(device_fns.create_image),
            vkDestroyImage:                          Some(device_fns.destroy_image),
            vkCmdCopyBuffer:                         Some(device_fns.cmd_copy_buffer),
            vkGetBufferMemoryRequirements2KHR:       Some(device_fns.get_buffer_memory_reqs2),
            vkGetImageMemoryRequirements2KHR:        Some(device_fns.get_image_memory_reqs2),
            vkBindBufferMemory2KHR:                  Some(device_fns.bind_buffer_memory2),
            vkBindImageMemory2KHR:                   Some(device_fns.bind_image_memory2),
            vkGetPhysicalDeviceMemoryProperties2KHR: Some(instance.inst_fns.get_gpu_memory_properties2),
            vkGetDeviceBufferMemoryRequirements:     Some(device_fns.get_device_buffer_memory_reqs),
            vkGetDeviceImageMemoryRequirements:      Some(device_fns.get_device_image_memory_reqs),
        };

        let mut vma_ci = api::VmaAllocatorCreateInfo::default();
        vma_ci.physicalDevice   = chosen_gpu.handle;
        vma_ci.device           = device_handle;
        vma_ci.instance         = instance.handle;
        vma_ci.flags            = vma_flags;
        vma_ci.pVulkanFunctions = &vma_fns;

        let mut vma_allocator: MaybeUninit<_> = MaybeUninit::<api::VmaAllocator>::uninit();
        call_throw!(api::vmaCreateAllocator, &vma_ci, vma_allocator.as_mut_ptr());

        //---------------------------------------------------------------------------------------//
        // (Finally) Return the Device
        println!("Finished created Vulkan Device.");

        return Device {
            fns:       device_fns,
            handle:    device_handle,
            allocator: unsafe { vma_allocator.assume_init() },
            instance,
            surface,
            gpus,
            gpu: chosen_gpu,
        };
    }

    pub fn destroy(&mut self) {
        call!(api::vmaDestroyAllocator, self.allocator);
        call!(self.fns.destroy_device, self.handle, ptr::null());
    }

    /// If None is passed as gpu_index, then the first available Discrete GPU is chosen.
    pub fn select_gpu(gpu_list: &Vec<Rc<Gpu>>, gpu_index: Option<usize>) -> Rc<Gpu> {
        assert!(gpu_list.len() > 0);

        if let Some(index) = gpu_index {
            if index < gpu_list.len() {
                return gpu_list[index].clone();
            }
        } else {
            for gpu in gpu_list {
                if (gpu.properties.deviceType & api::VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU) != 0 {
                    return gpu.clone();
                }
            }
        }

        println!("[WARN] Device::select_gpu :: Failed to find a discrete gpu. Falling back to the first available gpu.");
        return gpu_list[0].clone();
    }

    pub fn create_device_context(device: Rc<Self>) -> context::DeviceContext {
        return context::DeviceContext::new(device);
    }

    pub fn get_queue(&self, queue_type: util::QueueType) -> api::VkQueue {
        let queue_index = self.get_queue_index(queue_type);

        let mut queue: MaybeUninit<_> = MaybeUninit::<api::VkQueue>::uninit();
        call!(
            self.fns.get_queue,
            self.handle,
            queue_index,
            0,
            queue.as_mut_ptr()
        );

        return unsafe { queue.assume_init() };
    }

    pub fn create_swapchain(&self, old_swapchain: Option<&Swapchain>) -> Swapchain {
        let present_queue = self.get_queue(util::QueueType::Present);

        // Let's grab some data from the old swapchain
        let mut cached_width: u32 = 0;
        let mut cached_height: u32 = 0;

        let mut old_swapchain_handle: MaybeUninit<_> = MaybeUninit::<api::VkSwapchainKHR>::uninit();
        let mut new_swapchain_handle: MaybeUninit<_> = MaybeUninit::<api::VkSwapchainKHR>::uninit();

        if let Some(ref swapchain) = old_swapchain {
            cached_width = swapchain.cached_width;
            cached_height = swapchain.cached_height;

            old_swapchain_handle.write(swapchain.handle);

            for view in &swapchain.image_views {
                call!(self.fns.destroy_image_view, self.handle, *view, ptr::null_mut());
            }
        }

        // Don't allow swapchain dimensions less than 8.
        const MIN_SIZE: u32 = 8;
        cached_width = std::cmp::max(cached_width, MIN_SIZE);
        cached_width = std::cmp::max(cached_width, MIN_SIZE);

        let surface_format = self.gpu.select_surface_format(consts::DEVICE_FEATURES.prefer_hdr);
        let swapchain_caps = Gpu::query_swapchain_capabilities(&self.instance, &self.surface, self.gpu.handle);

        // Select the present mode
        let mut present_mode: api::VkPresentModeKHR = api::VK_PRESENT_MODE_FIFO_KHR; //worst-case fallback if mailbox is not present
        for mode in swapchain_caps.present_modes {
            if mode == api::VK_PRESENT_MODE_MAILBOX_KHR {
                present_mode = mode;
                break;
            }
        }

        // For the docs on the surface capabilities:
        // > currentExtent is the current width and height of the surface, or the special value (0xFFFFFFFF, 0xFFFFFFFF) indicating
        //   that the surface size will be determined by the extent of a swapchain targeting the surface.
        //
        // We'll use the cached dimensions as the fallback. This will either be set by the device recreation or by onResize()
        let mut swapchain_extent = api::VkExtent2D{ width: cached_width, height: cached_height };
        if (swapchain_caps.capabilities.currentExtent.width  != u32::MAX && swapchain_caps.capabilities.currentExtent.height != u32::MAX) {
            swapchain_extent = swapchain_caps.capabilities.currentExtent;
        }

        // Clamp to the value allowed by the GPU.
        let image_min: api::VkExtent2D = swapchain_caps.capabilities.minImageExtent;
        let image_max: api::VkExtent2D = swapchain_caps.capabilities.maxImageExtent;
        swapchain_extent.width  = swapchain_extent.width.clamp(image_min.width,  image_max.width);
        swapchain_extent.height = swapchain_extent.height.clamp(image_min.height, image_max.height);

        let mut image_count = swapchain_caps.capabilities.minImageCount + 1;
        if (swapchain_caps.capabilities.maxImageCount > 0 && image_count > swapchain_caps.capabilities.maxImageCount) {
            image_count = swapchain_caps.capabilities.maxImageCount;
        }

        assert!(image_count > 0); // double check that we aren't about to accidentally allow UINT32_MAX images
        let max_images_in_flight = image_count - 1;

        // Create the Swapchain
        //

        let mut swapchain_ci = api::VkSwapchainCreateInfoKHR::default();
        swapchain_ci.surface          = self.surface.handle;
        swapchain_ci.minImageCount    = image_count;
        swapchain_ci.imageFormat      = surface_format.format;
        swapchain_ci.imageColorSpace  = surface_format.colorSpace;
        swapchain_ci.imageExtent      = swapchain_extent;
        swapchain_ci.imageArrayLayers = 1;
        swapchain_ci.imageUsage       = api::VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT|api::VK_IMAGE_USAGE_TRANSFER_DST_BIT;

        // We expect to have a present and graphics queue.
        let present_queue_index  = self.gpu.queue_infos.present.expect("Failed to obtain present queue index");
        let graphics_queue_index = self.gpu.queue_infos.graphics.expect("Failed to obtain graphics queue index");

        // Setup the queue family indices
        let queue_family_indices: [u32; 2] = [present_queue_index, graphics_queue_index];

        if present_queue_index != graphics_queue_index {
            swapchain_ci.imageSharingMode      = api::VK_SHARING_MODE_CONCURRENT;
            swapchain_ci.queueFamilyIndexCount = 2;
            swapchain_ci.pQueueFamilyIndices   = queue_family_indices.as_ptr();
        } else {
            swapchain_ci.imageSharingMode      = api::VK_SHARING_MODE_EXCLUSIVE;
            swapchain_ci.queueFamilyIndexCount = 0;
            swapchain_ci.pQueueFamilyIndices   = ptr::null_mut();
        }

        swapchain_ci.preTransform   = swapchain_caps.capabilities.currentTransform;
        swapchain_ci.compositeAlpha = api::VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR;
        swapchain_ci.presentMode    = present_mode;
        swapchain_ci.clipped        = api::VK_TRUE;

        if old_swapchain.is_some() {
            swapchain_ci.oldSwapchain = unsafe { old_swapchain_handle.assume_init() };
        }

        call_throw!(self.fns.create_swapchain, self.handle, &swapchain_ci, std::ptr::null(), new_swapchain_handle.as_mut_ptr());
        let swapchain = unsafe { new_swapchain_handle.assume_init() };

        // Create Swapchain Images/Views
        //

        // We requested the image count before, now let's query in case the driver didn't like our request
        image_count = 0;
        call_throw!(self.fns.get_swapchain_images, self.handle, swapchain, &mut image_count, ptr::null_mut());

        let mut swapchain_images      = Vec::<api::VkImage>::with_capacity(image_count as usize);
        let mut swapchain_image_views = Vec::<api::VkImageView>::with_capacity(image_count as usize);

        swapchain_images.resize(image_count as usize, ptr::null_mut());
        call_throw!(self.fns.get_swapchain_images, self.handle, swapchain, &mut image_count, swapchain_images.as_mut_ptr());

        for i in 0..image_count {
            let mut image_view_ci = api::VkImageViewCreateInfo::default();
            image_view_ci.image                           = swapchain_images[i as usize];
            image_view_ci.viewType                        = api::VK_IMAGE_VIEW_TYPE_2D;
            image_view_ci.format                          = surface_format.format;
            image_view_ci.subresourceRange.aspectMask     = api::VK_IMAGE_ASPECT_COLOR_BIT;
            image_view_ci.subresourceRange.baseMipLevel   = 0;
            image_view_ci.subresourceRange.levelCount     = 1;
            image_view_ci.subresourceRange.baseArrayLayer = 0;
            image_view_ci.subresourceRange.layerCount     = 1;

            let mut view: MaybeUninit<_> = MaybeUninit::<api::VkImageView>::uninit();
            call_throw!(self.fns.create_image_view, self.handle, &image_view_ci, ptr::null(), view.as_mut_ptr());

            swapchain_image_views.push(unsafe{ view.assume_init() });
        }

        // Create Swapchain Frame Sync
        //

        let render_semaphores = if let Some(old) = old_swapchain {
            old.render_semaphores.clone()
        } else {
            let mut sems = Vec::<super::Semaphore>::with_capacity(swapchain_images.len());
            for i in 0..swapchain_images.len() {
                sems.push(self.create_semaphore());
            }
            sems
        };

        let present_semaphores = if let Some(old) = old_swapchain {
            old.present_semaphores.clone()
        } else {
            let mut sems = Vec::<super::Semaphore>::with_capacity(swapchain_images.len());
            for i in 0..swapchain_images.len() {
                sems.push(self.create_semaphore());
            }
            sems
        };

        let render_fences = if let Some(old) = old_swapchain {
            old.render_fences.clone()
        } else {
            let mut sems = Vec::<super::Fence>::with_capacity(swapchain_images.len());
            for i in 0..swapchain_images.len() {
                // Create the fence in a signaled state, indicating that the first frame has already been "rendered".
                // This will prevent the application from waiting indefinitely for the first frame to render since it
                // cannot be rendered until a frame is "rendered" before it.
                sems.push(self.create_fence(true));
            }
            sems
        };

        // Create Swapchain Function Table
        //

        let swapchain_fns = SwapchainFnTable{};

        return Swapchain{
            fns:                swapchain_fns,
            handle:             swapchain,
            present_queue,
            image_views:        swapchain_image_views,
            images:             swapchain_images,
            present_semaphores,
            render_semaphores,
            render_fences,
            frame_index:        0,
            swapchain_index:    0,
            cached_width:       swapchain_extent.width,
            cached_height:      swapchain_extent.height,
            known_generation:   if let Some(old) = old_swapchain { old.known_generation   } else { 0 },
            current_generation: if let Some(old) = old_swapchain { old.current_generation } else { 0 },
        };
    }

    pub fn destroy_swapchain(&self, swapchain: &mut Swapchain) {
        //@assume: vkDeviceWaitIdle has already been called.

        for view in &swapchain.image_views {
            call!(self.fns.destroy_image_view, self.handle, *view, ptr::null_mut());
        }

        for sem in &swapchain.present_semaphores {
            self.destroy_semaphore(sem);
        }

        for sem in &swapchain.render_semaphores {
            self.destroy_semaphore(sem);
        }

        for fence in &swapchain.render_fences {
            self.destroy_fence(&fence);
        }

        call!(self.fns.destroy_swapchain, self.handle, swapchain.handle, ptr::null());
    }

    pub fn get_queue_index(&self, queue_type: util::QueueType) -> u32 {
        let mut queue_index: u32 = 0;
        match queue_type {
            util::QueueType::Present => {
                if let Some(index) = self.gpu.queue_infos.present {
                    queue_index = index;
                } else {
                    panic!("Failed to find Present Queue. This is a fatal error.");
                }
            }
            util::QueueType::Graphics => {
                if let Some(index) = self.gpu.queue_infos.graphics {
                    queue_index = index;
                } else {
                    panic!("Failed to find Graphics Queue. This is a fatal error.");
                }
            }
            util::QueueType::Compute => {
                if let Some(index) = self.gpu.queue_infos.compute {
                    queue_index = index;
                } else {
                    // Failed to find compute index, let's fallback to the graphics queue
                    if let Some(index) = self.gpu.queue_infos.graphics {
                        queue_index = index;
                    } else {
                        panic!("Failed to find Graphics Queue. This is a fatal error.");
                    }
                }
            }
            util::QueueType::Transfer => {
                if let Some(index) = self.gpu.queue_infos.transfer {
                    queue_index = index;
                } else {
                    // Failed to find the transfer index, let's fallback to the graphics queue
                    if let Some(index) = self.gpu.queue_infos.graphics {
                        queue_index = index;
                    } else {
                        panic!("Failed to find Graphics Queue. This is a fatal error.");
                    }
                }
            }
        }

        return queue_index;
    }

    pub fn create_command_pool(&self, queue_type: util::QueueType) -> CommandPool {
        let queue_index = self.get_queue_index(queue_type);

        let mut command_pool_ci = api::VkCommandPoolCreateInfo::default();
        command_pool_ci.queueFamilyIndex = queue_index;

        let mut pool: MaybeUninit<_> = MaybeUninit::<api::VkCommandPool>::uninit();
        call_throw!(self.fns.create_command_pool, self.handle, &command_pool_ci, ptr::null(), pool.as_mut_ptr());

        let fn_table = CommandPoolFnTable{};

        return CommandPool{
            fns:    fn_table,
            handle: unsafe { pool.assume_init() },
        };
    }

    pub fn destroy_command_pool(&self, command_pool: &mut CommandPool) {
        call!(self.fns.destroy_command_pool, self.handle, command_pool.handle, ptr::null_mut());
    }

    pub fn create_command_buffer(&self, command_pool: &CommandPool) -> CommandBuffer {
        let mut command_buffer_ci = api::VkCommandBufferAllocateInfo::default();
        command_buffer_ci.commandPool        = command_pool.handle;
        command_buffer_ci.commandBufferCount = 1;

        let mut buffer: MaybeUninit<_> = MaybeUninit::<api::VkCommandBuffer>::uninit();
        call_throw!(self.fns.alloc_command_buffers, self.handle, &command_buffer_ci, buffer.as_mut_ptr());

        let fn_table = CommandBufferFnTable{
            begin_command_buffer:  self.fns.begin_command_buffer,
            end_command_buffer:    self.fns.end_command_buffer,
            reset_command_buffer:  self.fns.reset_command_buffer,
            cmd_pipeline_barrier2: self.fns.cmd_pipeline_barrier2,
            cmd_clear_color_image: self.fns.cmd_clear_color_image,
            cmd_blit_image2:       self.fns.cmd_blit_image2,
        };

        return CommandBuffer::new(fn_table, unsafe { buffer.assume_init() });
    }

    pub fn create_semaphore(&self) -> super::Semaphore {
        let semaphore_ci = api::VkSemaphoreCreateInfo::default();

        let mut semaphore: MaybeUninit<_> = MaybeUninit::<api::VkSemaphore>::uninit();
        call_throw!(self.fns.create_semaphore, self.handle, &semaphore_ci, ptr::null(), semaphore.as_mut_ptr());

        return unsafe { semaphore.assume_init() };
    }

    pub fn create_timeline_semaphore(&self, initial_value: u64) -> super::TimelineSemaphore {
        let mut timeline_ci = api::VkSemaphoreTypeCreateInfo::default();
        timeline_ci.initialValue = initial_value;

        let timeline_ci_ptr: *mut api::VkSemaphoreTypeCreateInfo = &mut timeline_ci;

        let mut semaphore_ci = api::VkSemaphoreCreateInfo::default();
        semaphore_ci.pNext = timeline_ci_ptr as *mut std::os::raw::c_void;

        let mut semaphore: MaybeUninit<_> = MaybeUninit::<api::VkSemaphore>::uninit();
        call_throw!(self.fns.create_semaphore, self.handle, &semaphore_ci, ptr::null(), semaphore.as_mut_ptr());

        return unsafe { semaphore.assume_init() };
    }

    pub fn destroy_semaphore(&self, semaphore: &super::Semaphore) {
        call!(self.fns.destroy_semaphore, self.handle, *semaphore, ptr::null());
    }

    pub fn destroy_timeline_semaphore(&self, semaphore: &super::TimelineSemaphore) {
        self.destroy_semaphore(semaphore);
    }

    pub fn create_fence(&self, set_signaled: bool) -> super::Fence {
        let mut fence_ci = api::VkFenceCreateInfo::default();
        fence_ci.flags = if set_signaled { api::VK_FENCE_CREATE_SIGNALED_BIT } else { 0 };

        let mut fence: MaybeUninit<_> = MaybeUninit::<api::VkFence>::uninit();
        call_throw!(self.fns.create_fence, self.handle, &fence_ci, ptr::null(), fence.as_mut_ptr());

        return unsafe { fence.assume_init() };
    }

    pub fn destroy_fence(&self, fence: &super::Fence) {
        call!(self.fns.destroy_fence, self.handle, *fence, ptr::null());
    }

    pub fn wait_idle(&self) {
        call!(self.fns.wait_idle, self.handle);
    }

    pub fn allocate_image_memory(
        &self,
        extent:             api::VkExtent3D,
        format:             api::VkFormat,
        image_usage:        api::VkImageUsageFlags,
        memory_usage:       api::VmaMemoryUsage,
        memory_props:       api::VkMemoryPropertyFlagBits,
        image_aspect_flags: api::VkImageAspectFlagBits)    -> super::AllocatedImage
    {
        let mut result = super::AllocatedImage::default();

        //hardcoding the draw format to 32 bit float
        result.format = format;
        result.dims   = extent;

        let image_ci = util::make_image_ci(format, image_usage, extent);

        //for the draw image, we want to allocate it from gpu local memory
        let mut image_alloc_info = api::VmaAllocationCreateInfo::default();
        image_alloc_info.usage          = memory_usage;
        image_alloc_info.preferredFlags = memory_props;

        //allocate and create the image
        call_throw!(api::vmaCreateImage, self.allocator, &image_ci, &image_alloc_info, &mut result.image, &mut result.memory, ptr::null_mut());

        //build an image-view for the draw image to use for rendering
        let image_view_ci = util::make_image_view_ci(result.format, result.image, api::VK_IMAGE_ASPECT_COLOR_BIT);

        call_throw!(self.fns.create_image_view, self.handle, &image_view_ci, ptr::null_mut(), &mut result.view);

        return result;
    }

    pub fn destroy_image_memory(&self, image: &mut super::AllocatedImage) {
        call!(self.fns.destroy_image_view, self.handle, image.view, ptr::null_mut());
        call!(api::vmaDestroyImage, self.allocator, image.image, image.memory);

        image.image  = ptr::null_mut();
        image.memory = ptr::null_mut();
    }
}

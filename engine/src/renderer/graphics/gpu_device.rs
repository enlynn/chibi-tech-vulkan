
use crate::renderer::graphics::gpu_utils::call_throw;
use crate::window::NativeSurface;
use crate::util::ffi::*;

use super::api;
use super::consts;
use super::gpu_utils as util;

use std::borrow::Borrow;
use std::ffi::{CString, CStr};
use std::mem::MaybeUninit;
use std::ptr;
use std::rc::Rc;

//
// TODO:
// - Device Extensions require modifying 2 locations (is device valid, and device creation), which is not great. centralize this into an array.
//

pub struct Features {}

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
pub struct GpuQueueFamilies
{
    graphics: Option<QueueFamily>,
    present:  Option<QueueFamily>,
    compute:  Option<QueueFamily>,
    transfer: Option<QueueFamily>,
}

#[derive(Default)]
pub struct SwapchainSupportInfo
{
    capabilities:  api::VkSurfaceCapabilitiesKHR,
    formats:       Vec<api::VkSurfaceFormatKHR>,
    present_modes: Vec<api::VkPresentModeKHR>,
}

pub struct Gpu {
    pub handle:                             api::VkPhysicalDevice,
    pub properties:                         api::VkPhysicalDeviceProperties,
    pub features:                           api::VkPhysicalDeviceFeatures,
    pub memory_properties:                  api::VkPhysicalDeviceMemoryProperties,
    pub queue_infos:                        GpuQueueFamilies,
    pub swapchain_support_info:             SwapchainSupportInfo,
    pub supports_device_local_host_visible: bool,
    // supported swapchain surface format
    //surface_format: VkSurfaceFormatKHR;
    // supported swapchain depth format
    //depth_format: VkFormat;
}

pub struct Display {
    //todo:
}

pub struct Surface {
    handle: api::VkSurfaceKHR,
}

pub struct SwapchainImage {

}

pub struct Swapchain {
    images: [SwapchainImage; consts::MAX_BUFFERED_FRAMES],
}

pub struct Device {
    fns:       util::DeviceFnTable,
    handle:    api::VkDevice,

    instance:  Instance,
    surface:   Surface,
    //swapchain: Swapchain,

    gpus:     Vec<Rc<Gpu>>,
    //displays: Vec<Rc<Display>>,

    gpu:     Rc<Gpu>,
    //display: Rc<Display>,
}

unsafe extern "C" fn debug_callback(
    severity:      api::VkDebugUtilsMessageSeverityFlagBitsEXT,
    _message_type: api::VkDebugUtilsMessageTypeFlagsEXT,
    data:         *const api::VkDebugUtilsMessengerCallbackDataEXT,
    _:            *mut std::os::raw::c_void,
) -> api::VkBool32
{
    if (severity & api::VK_DEBUG_UTILS_MESSAGE_SEVERITY_VERBOSE_BIT_EXT) != 0 {
        println!("[VERBOSE]: {:?}", CStr::from_ptr((*data).pMessage));
    }
    else if (severity & api::VK_DEBUG_UTILS_MESSAGE_SEVERITY_INFO_BIT_EXT) != 0
    {
        println!("[INFO]: {:?}", CStr::from_ptr((*data).pMessage));
    }
    else if (severity & api::VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT) != 0
    {
        println!("[WARNING]: {:?}", CStr::from_ptr((*data).pMessage));
    }
    else if (severity & api::VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT) != 0
    {
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
        let desired_validation_layer = byte_array_as_cstr!(consts::VK_LAYER_KHRONOS_VALIDATION_LAYER_NAME);

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

        let severities =
            api::VK_DEBUG_UTILS_MESSAGE_SEVERITY_WARNING_BIT_EXT |
            api::VK_DEBUG_UTILS_MESSAGE_SEVERITY_ERROR_BIT_EXT;

        let message_types =
            api::VK_DEBUG_UTILS_MESSAGE_TYPE_GENERAL_BIT_EXT     |
            api::VK_DEBUG_UTILS_MESSAGE_TYPE_PERFORMANCE_BIT_EXT |
            api::VK_DEBUG_UTILS_MESSAGE_TYPE_VALIDATION_BIT_EXT;

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
        let mut surface_ext_found          = false;
        let mut platform_surface_ext_found = false;

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
        util::call_throw!(global_fns.create_instance, &instance_ci as *const _, std::ptr::null(), &mut instance as *mut _);

        // Load Instance-level Functions
        //
        let instance_fns: util::InstanceFnTable = match util::load_instance_functions(&global_fns, instance) {
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
            }
            else {
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

        Ok(Instance{
            glb_fns:              global_fns,
            inst_fns:             instance_fns,
            handle:               instance,
            requested_layers:     instance_layer_strings,
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
                    sType:   api::VK_STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR,
                    pNext:   std::ptr::null(),
                    flags:   0,
                    display: native.display as *mut api::wl_display,
                    surface: native.surface as *mut api::wl_surface,
                };

                let mut surf = MaybeUninit::<api::VkSurfaceKHR>::uninit();
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
                    dpy:    native.display as *mut api::Display,
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
            }
            else {
                panic!("Invalid native surface for linux.");
            }
        } else {
            Err("unsupported operating system".to_string())
        };

        result
    }
}

impl Gpu {
    fn get_queue_families(instance: &Instance, surface: &Surface, gpu: api::VkPhysicalDevice) -> GpuQueueFamilies {
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
            util::call_throw!(instance.inst_fns.get_gpu_surface_support, gpu, queue_family_index, surface.handle, &mut supports_present);

            if supports_present == api::VK_TRUE {
                result.present = Some(queue_family_index);
            }

            queue_family_index += 1;
        }

        return result;
    }

    pub fn query_swapchain_capabilities(instance: &Instance, surface: &Surface, gpu: api::VkPhysicalDevice) -> SwapchainSupportInfo {
        // Surface capabilities
        let mut capabilities_unsafe = MaybeUninit::<api::VkSurfaceCapabilitiesKHR>::uninit();
        util::call_throw!(instance.inst_fns.get_gpu_surface_capabilities, gpu, surface.handle, capabilities_unsafe.as_mut_ptr());

        // Surface formats
        let formats = instance.inst_fns.enumerate_gpu_surface_formats(gpu, surface.handle);

        // Present modes
        let present_modes = instance.inst_fns.enumerate_gpu_present_modes(gpu, surface.handle);

        return SwapchainSupportInfo{ capabilities: unsafe { capabilities_unsafe.assume_init() }, formats, present_modes };
    }

    fn does_gpu_meet_requirements(instance: &Instance, surface: &Surface, gpu: api::VkPhysicalDevice, gpu_features: &api::VkPhysicalDeviceFeatures) -> bool {
        let queue_families = Self::get_queue_families(instance, surface, gpu);

        let has_present  = queue_families.present.is_some();
        let has_graphics = queue_families.graphics.is_some();
        let has_transfer = queue_families.transfer.is_some();
        let has_compute  = queue_families.compute.is_some();
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
            } else if ext_c_str == byte_array_as_cstr!(api::VK_KHR_TIMELINE_SEMAPHORE_EXTENSION_NAME) {
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
            call!(instance.inst_fns.get_gpu_properties, gpu, properties_unsafe.as_mut_ptr());

            let mut features_unsafe = MaybeUninit::<api::VkPhysicalDeviceFeatures>::uninit();
            call!(instance.inst_fns.get_gpu_features, gpu, features_unsafe.as_mut_ptr());

            let mut memory_unsafe = MaybeUninit::<api::VkPhysicalDeviceMemoryProperties>::uninit();
            call!(instance.inst_fns.get_gpu_memory_properties, gpu, memory_unsafe.as_mut_ptr());

            // let's unwrap the types
            let properties = unsafe { properties_unsafe.assume_init() };
            let features   = unsafe { features_unsafe.assume_init()   };
            let memory     = unsafe { memory_unsafe.assume_init()     };

            // Check if device supports local/host visible combo
            let mut supports_device_local_host_visible = false;
            for i in 0..memory.memoryTypeCount {
                let has_host_visible = (memory.memoryTypes[i as usize].propertyFlags & api::VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT) != 0;
                let has_device_local = (memory.memoryTypes[i as usize].propertyFlags & api::VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT) != 0;
                if has_host_visible && has_device_local {
                    supports_device_local_host_visible = true;
                    break;
                }
            }

            if !Gpu::does_gpu_meet_requirements(instance, surface, gpu, &features) {
                continue;
            }

            let adapter = Rc::new(Gpu{
                handle:                             gpu,
                properties,
                features,
                memory_properties:                  memory,
                queue_infos:                        Gpu::get_queue_families(instance, surface, gpu),
                swapchain_support_info:             Gpu::query_swapchain_capabilities(instance, surface, gpu),
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
}

impl Device {
    pub fn new(create_info: CreateInfo) -> Device {
        let instance = match Instance::new(create_info.software_version, create_info.software_name.as_str()) {
            Ok(inst)     => inst,
            Err(reason)  => panic!("Failed to create vulkan instance: {}", reason),
        };

        let surface = match Surface::new(&instance, create_info.surface) {
            Ok(surf)     => surf,
            Err(reason)  => panic!("Failed to create vulkan surface: {}", reason),
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
            queue_ci[i as usize].queueCount       = 1;
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

        let feature_sync2_ptr: *mut api::VkPhysicalDeviceSynchronization2Features = &mut feature_sync2;

        let mut feature_timeline = api::VkPhysicalDeviceTimelineSemaphoreFeatures::default();
        feature_timeline.timelineSemaphore = api::VK_TRUE;
        feature_timeline.pNext = feature_sync2_ptr as *mut c_void;

        let feature_timeline_ptr: *mut api::VkPhysicalDeviceTimelineSemaphoreFeatures = &mut feature_timeline;

        //let enabled_features = api::VkPhysicalDeviceFeatures::default();
        // left here in case I want to override the defaults in the future

        let mut enabled_features2 = api::VkPhysicalDeviceFeatures2{
            pNext: feature_timeline_ptr as *mut c_void,
            ..Default::default()
        };

        let enabled_features2_ptr: *mut api::VkPhysicalDeviceFeatures2 = &mut enabled_features2;

        // 3. Build the list of device extensions

        let mut extension_list_strings = Vec::<CString>::with_capacity(3);
        let mut extension_list         = Vec::<*const std::os::raw::c_char>::with_capacity(3);

        let swapchain_ext_string: CString = byte_array_as_cstr!(api::VK_KHR_SWAPCHAIN_EXTENSION_NAME).into();
        let semaphore_ext_string: CString = byte_array_as_cstr!(api::VK_KHR_TIMELINE_SEMAPHORE_EXTENSION_NAME).into();

        extension_list.push(swapchain_ext_string.as_ptr());
        extension_list.push(semaphore_ext_string.as_ptr());

        extension_list_strings.push(swapchain_ext_string);
        extension_list_strings.push(semaphore_ext_string);

        if chosen_gpu.require_portability_subset(&instance) {
            let portability_ext_string: CString = byte_array_as_cstr!(consts::VK_KHR_PORTABILITY_SUBSET_EXTENSION_NAME).into();
            extension_list.push(portability_ext_string.as_ptr());
            extension_list_strings.push(portability_ext_string);
        }

        // 4. Create the Device!

        let mut device_ci = api::VkDeviceCreateInfo::default();
        device_ci.queueCreateInfoCount    = queue_count;
        device_ci.pQueueCreateInfos       = queue_ci.as_ptr();
        device_ci.enabledExtensionCount   = extension_list.len() as u32;
        device_ci.ppEnabledExtensionNames = extension_list.as_mut_ptr();
        device_ci.pNext                   = enabled_features2_ptr as *mut c_void;

        let mut device_handle: api::VkDevice = std::ptr::null_mut();
        call_throw!(instance.inst_fns.create_device, chosen_gpu.handle, &device_ci as *const _, ptr::null(), &mut device_handle as *mut _);

        //---------------------------------------------------------------------------------------//
        // Load Device Functions

        let device_fns = util::load_device_functions(&instance.glb_fns, instance.handle, device_handle).expect("Failed to load Vulkan Device level functions");

        //---------------------------------------------------------------------------------------//
        // TODO:
        // - Get the Command Queues
        // - Create the Swapchain
        // - Load the Vulkan Memory Allocator

        //---------------------------------------------------------------------------------------//
        // (Finally) Return the Device

        return Device{
            fns: device_fns,
            handle: device_handle,
            instance,
            surface,
            gpus,
            gpu: chosen_gpu,
        };
    }

    /// If None is passed as gpu_index, then the first available Discrete GPU is chosen.
    pub fn select_gpu(gpu_list: &Vec<Rc<Gpu>>, gpu_index: Option<usize>) -> Rc<Gpu> {
        assert!(gpu_list.len() > 0);

        if let Some(index) = gpu_index {
            if index < gpu_list.len() {
                return gpu_list[index].clone();
            }
        }
        else
        {
            for gpu in gpu_list {
                if (gpu.properties.deviceType & api::VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU) != 0
                {
                    return gpu.clone();
                }
            }
        }

        println!("[WARN] Device::select_gpu :: Failed to find a discrete gpu. Falling back to the first available gpu.");
        return gpu_list[0].clone();
    }
}

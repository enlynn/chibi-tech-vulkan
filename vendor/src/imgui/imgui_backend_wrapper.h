#ifndef _IMGUI_BACKEND_WRAPPER_H_
#define _IMGUI_BACKEND_WRAPPER_H_

// GLFW Platform wrapper
#include "cpp_backend/imgui_impl_glfw.h"

// Vulkan Platform wrapper
#define IMGUI_IMPL_VULKAN_NO_PROTOTYPES
#define IMGUI_IMPL_VULKAN_HAS_DYNAMIC_RENDERING
#include "cpp_backend/imgui_impl_vulkan.h"

extern "C" void ig_load_vk_functions(PFN_vkGetInstanceProcAddr get_instance_proc_addr, VkInstance instance);

#endif //_IMGUI_BACKEND_WRAPPER_H_

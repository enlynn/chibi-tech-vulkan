#include "imgui_backend_wrapper.h"

#include "cpp_backend/imgui_impl_glfw.cpp"
#include "cpp_backend/imgui_impl_vulkan.cpp"

#include "../vulkan/cpp/vulkan/vulkan.h"
#include <vulkan/vulkan_core.h>

extern "C" void ig_load_vk_functions(PFN_vkGetInstanceProcAddr get_instance_proc_addr, VkInstance instance) {
    struct UserData {
        PFN_vkGetInstanceProcAddr vkGetInstanceProcAddress;
        VkInstance                instance;
    };

    UserData data = {get_instance_proc_addr, instance};
    ImGui_ImplVulkan_LoadFunctions([](const char* function_name, void* user_data) {
        UserData* data = static_cast<UserData*>(user_data);
        return data->vkGetInstanceProcAddress(data->instance, function_name);
    }, &data);
}

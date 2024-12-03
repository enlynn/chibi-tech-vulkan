
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

include!("imgui_bindings.rs");

impl Default for ImVec2 {
    fn default() -> Self {
        Self{ x: 0.0, y: 0.0 }
    }
}

// Backend wrapper
use crate::vulkan;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ImGuiVulkanInitInfo {
    pub Instance:                    vulkan::VkInstance,
    pub PhysicalDevice:              vulkan::VkPhysicalDevice,
    pub Device:                      vulkan::VkDevice,
    pub QueueFamily:                 u32,
    pub Queue:                       vulkan::VkQueue,
    pub DescriptorPool:              vulkan::VkDescriptorPool,         // See requirements in note above
    pub RenderPass:                  vulkan::VkRenderPass,             // Ignored if using dynamic rendering
    pub MinImageCount:               u32,                              // >= 2
    pub ImageCount:                  u32,                              // >= MinImageCount
    pub MSAASamples:                 vulkan::VkSampleCountFlagBits,    // 0 defaults to VK_SAMPLE_COUNT_1_BIT

    // (Optional)
    pub PipelineCache:               vulkan::VkPipelineCache,
    pub Subpass:                     u32,

    // (Optional) Dynamic Rendering
    // Need to explicitly enable VK_KHR_dynamic_rendering extension to use this, even for Vulkan 1.3.
    pub UseDynamicRendering:         bool,
    pub PipelineRenderingCreateInfo: vulkan::VkPipelineRenderingCreateInfoKHR,

    // (Optional) Allocation, Debugging
    pub Allocator:                   *const vulkan::VkAllocationCallbacks,
    pub CheckVkResultFn:             Option<extern "C" fn(err: vulkan::VkResult)>,
    pub MinAllocationSize:           vulkan::VkDeviceSize,      // Minimum allocation size. Set to 1024*1024 to satisfy zealous best practices validation layer and waste a little memory.
}

extern "C" {
    // Imgui-Glfw Backend Bindings
    //

    /// bool ImGui_ImplGlfw_InitForOpenGL(GLFWwindow* window, bool install_callbacks);
    fn ImGui_ImplGlfw_InitForVulkan(window: *mut crate::glfw::GLFWwindow, install_callbacks: bool) -> bool;
    /// bool ImGui_ImplGlfw_Shutdown();
    fn ImGui_ImplGlfw_Shutdown();
    /// void ImGui_ImplGlfw_NewFrame();
    fn ImGui_ImplGlfw_NewFrame();

    // Imgui-Vulkan Backend Bindings
    //

    /// bool ImGui_ImplVulkan_Init(ImGui_ImplVulkan_InitInfo* info);
    fn ImGui_ImplVulkan_Init(info: *mut ImGuiVulkanInitInfo) -> bool;
    /// void ImGui_ImplVulkan_Shutdown();
    fn ImGui_ImplVulkan_Shutdown();
    /// void ImGui_ImplVulkan_NewFrame();
    fn ImGui_ImplVulkan_NewFrame();
    /// void ImGui_ImplVulkan_RenderDrawData(ImDrawData* draw_data, VkCommandBuffer command_buffer, VkPipeline pipeline = VK_NULL_HANDLE);
    fn ImGui_ImplVulkan_RenderDrawData(draw_data: *const ImDrawData, command_buffer: vulkan::VkCommandBuffer, pipeline: vulkan::VkPipeline);
    /// bool ImGui_ImplVulkan_CreateFontsTexture();
    fn ImGui_ImplVulkan_CreateFontsTexture();
    /// void ImGui_ImplVulkan_DestroyFontsTexture();
    fn ImGui_ImplVulkan_DestroyFontsTexture();
    /// void ImGui_ImplVulkan_SetMinImageCount(uint32_t min_image_count); // To override MinImageCount after initialization (e.g. if swap chain is recreated)
    fn ImGui_ImplVulkan_SetMinImageCount(min_image_count: u32);

    //extern "C" IMGUI_IMPL_API bool ImGui_ImplVulkan_LoadFunctions(PFN_vkVoidFunction(*loader_func)(const char* function_name, void* user_data), void* user_data = nullptr);
    fn ig_load_vk_functions(get_instance_proc_addr: vulkan::FN_vkGetInstanceProcAddr, instance: vulkan::VkInstance);
}

pub fn ig_load_vulkan_functions(get_instance_proc_addr: vulkan::FN_vkGetInstanceProcAddr, instance: vulkan::VkInstance) {
    unsafe { ig_load_vk_functions(get_instance_proc_addr, instance) };
}

pub fn ig_glfw_init(window: *mut crate::glfw::GLFWwindow, install_callbacks: bool) -> bool {
    return unsafe { ImGui_ImplGlfw_InitForVulkan(window, install_callbacks) };
}

pub fn ig_glfw_shutdown() {
    return unsafe { ImGui_ImplGlfw_Shutdown() };
}

pub fn ig_glfw_new_frame() {
    return unsafe { ImGui_ImplGlfw_NewFrame() };
}

pub fn ig_vulkan_init(mut info: ImGuiVulkanInitInfo) -> bool {
    return unsafe { ImGui_ImplVulkan_Init(&mut info as *mut ImGuiVulkanInitInfo) };
}

pub fn ig_vulkan_shutdown() {
    unsafe { ImGui_ImplVulkan_Shutdown() };
}

pub fn ig_vulkan_new_frame() {
    unsafe { ImGui_ImplVulkan_NewFrame() };
}

pub fn ig_vulkan_render_draw_data(command_buffer: vulkan::VkCommandBuffer, pipeline: vulkan::VkPipeline) {

    unsafe { ImGui_ImplVulkan_RenderDrawData(igGetDrawData(), command_buffer, pipeline) };
}

pub fn ig_vulkan_create_fonts_texture() {
    unsafe { ImGui_ImplVulkan_CreateFontsTexture() };
}

pub fn ig_vulkan_destroy_fonts_texture() {
    unsafe { ImGui_ImplVulkan_DestroyFontsTexture() };
}

pub fn ig_vulkan_set_min_image_count(min_image_count: u32) {
    unsafe { ImGui_ImplVulkan_SetMinImageCount(min_image_count) };
}

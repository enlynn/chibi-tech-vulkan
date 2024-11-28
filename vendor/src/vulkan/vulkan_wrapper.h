#ifndef _VULKAN_WRAPPER_H_
#define _VULKAN_WRAPPER_H_

// we'll manually load vulkan functions
#define VK_NO_PROTOTYPES

#ifdef __linux__
#  define VK_USE_PLATFORM_WAYLAND_KHR 1
#  define VK_USE_PLATFORM_XCB_KHR     1
#  define VK_USE_PLATFORM_XLIB_KHR    1
#elif defined(_WIN32) || defined(WIN32)
#  define VK_USE_PLATFORM_WIN32_KHR   1
#else
# error Unsupported platform for vulkan bindings.
#endif


//#define VMA_IMPLEMENTATION

#include "cpp/vulkan/vulkan.h"
#include "cpp/vma/vk_mem_alloc.h"


#endif//_VULKAN_WRAPPER_H_

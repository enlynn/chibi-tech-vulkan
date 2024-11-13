#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]

include!(concat!(env!("OUT_DIR"), "/vulkan_bindings.rs"));

pub const VK_API_VERSION_1_1: u32 = 1u32 << 22u32 | 1u32 << 12u32;
pub const VK_API_VERSION_1_2: u32 = 1u32 << 22u32 | 2u32 << 12u32;
pub const VK_API_VERSION_1_3: u32 = 1u32 << 22u32 | 3u32 << 12u32;

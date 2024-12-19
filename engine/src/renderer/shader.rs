use std::str::FromStr;

use common::math::{
    float4::*,
    float4x4::*,
};

use super::graphics::{
    *,
    gpu_device::*,
};

use vendor::vulkan::*;

// Shader Interop Structs
//

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub(crate) struct ComputePushConstants {
    pub data1: Float4,
    pub data2: Float4,
    pub data3: Float4,
    pub data4: Float4,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub(crate) struct GlobalSceneData {
    pub view:           Float4x4,
    //----------------- 16-byte boundary
    pub proj:           Float4x4,
    //----------------- 16-byte boundary
    pub view_proj:      Float4x4,
    //----------------- 16-byte boundary
    pub ambient_color:  Float4,
    pub sunlight_dir:   Float4,
    pub sunlight_color: Float4,
    pub padding0:       Float4,
    //----------------- 16-byte boundary
}

// vertex_buffer:       VkDeviceAddress,
// material_index: VkDeviceAddress,
//
//

// -> Dynamic Uniform Buffer
#[repr(C)]
pub(crate) struct GpuMeshUniform {
    pub transform: Float4x4,
}

// -> Dynamic Uniform Buffer
#[repr(C)]
pub(crate) struct GpuMaterialUniform {
    ambient_color: Float4,
    //----------------- 16-byte boundary
}

#[repr(C)]
pub(crate) struct GpuDrawPushConstants {
    //----------------- 16-byte boundary
    pub vertex_buffer:    VkDeviceAddress,
	pub mesh_data_buffer: VkDeviceAddress,
	//----------------- 16-byte boundary
	//material_buffer:  VkDeviceAddress,
	pub mesh_index:       u32,
	//material_index:   u32,
	//----------------- 16-byte boundary
}

pub(crate) enum ShaderStage {
    Vertex,
    Fragment,
    Compute,
}

// Shader Helper Functions
//

impl Default for GlobalSceneData {
    fn default() -> Self {
        Self{
            view:           Float4x4::identity(),
            proj:           Float4x4::identity(),
            view_proj:      Float4x4::identity(),
            ambient_color:  Float4::zero(),
            sunlight_dir:   Float4::zero(),
            sunlight_color: Float4::zero(),
            padding0:       Float4::zero(),
        }
    }
}

pub fn load_shader_module(device: &Device, shader_name: &str, stage: ShaderStage) -> VkShaderModule {
    use crate::core::asset_system::{AssetDrive, AssetSystem};
    use std::io::prelude::*;
    use std::fs::File;

    let asset_dir  = AssetSystem::get_root_dir(AssetDrive::Priv);

    //todo: cache this so we don't have to recreate it for every shader
    let shader_dir  = asset_dir.join("shaders/.cache");

    let mut shader_name_str = String::from_str(shader_name).expect("Failed to construct string.");
    match stage {
        ShaderStage::Vertex   => { shader_name_str.push_str(".vert.spv"); },
        ShaderStage::Fragment => { shader_name_str.push_str(".frag.spv"); },
        ShaderStage::Compute  => { shader_name_str.push_str(".comp.spv"); },
    };

    let shader_file = shader_dir.join(shader_name_str);
    let display = shader_file.display();

    println!("Shader Cache Directory: {:?}", shader_file);

    // let's read the file
    let mut file = match File::open(&shader_file) {
        Err(why) => panic!("couldn't open {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut file_data = Vec::<u8>::new();
    match file.read_to_end(&mut file_data) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_)    => {},
    }

    device.create_shader_module(file_data.as_slice()).expect("Failed to create VkShaderModule from gradient.spv")
}

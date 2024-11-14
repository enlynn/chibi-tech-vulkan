mod graphics;

use graphics::*;
use super::window::NativeSurface;

#[derive(Clone, Copy)]
pub enum RenderCommand{
    // Mesh-related commands
    CreateMesh,
    DestroyMesh,
    HideMesh,
    ShowMesh,

    // Texture-related commands
    CreateTexture,
    DestroyTexture,

    // Material-related commands
    CreateMaterial,
    DestroyMaterial,
}

pub struct RenderCommandBuffer{
    commands: Vec<RenderCommand>,
}

pub struct RendererCreateInfo {
    pub surface: NativeSurface,
}

pub struct RenderSystem{
    device: gpu_device::Device,
}

impl RenderSystem {
    pub fn new(create_info: RendererCreateInfo) -> RenderSystem {
        return RenderSystem{
            device: gpu_device::Device::new(gpu_device::CreateInfo{
                features: gpu_device::Features{},
                surface: create_info.surface,
            })
        };
    }

    pub fn render(&self, _command_buffer: RenderCommandBuffer) {

    }
}

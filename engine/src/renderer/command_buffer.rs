use std::collections::VecDeque;

use common::math::{float3::Float3, float4x4::*};
use assetlib::mesh::Vertex;

#[derive(Clone, Copy, PartialEq)]
pub enum RenderId {
    Unknown,                                                // Unknown Id - this is an error state
    None,                                                   // Generic Empty Id - this is a valid state
    Mesh(super::mesh::MeshId),
    Texture2d(super::texture::TextureId),
    Material(super::material::MaterialId),
    MaterialInstance(super::material::MaterialInstanceId),
}

pub struct CreateMeshInfo {
    pub vertices:     *const Vertex,
    pub vertex_count: usize,

    pub indices:      *const u32,
    pub index_count:  usize,

    //todo: other mesh properties
    pub transform:    Float4x4,

    pub material:     RenderId,

    // Some engine-id so that we can tell the engine the mesh has uploaded
    pub engine_id:    u64,
}

// Required for sending a *const Vertex
unsafe impl Send for CreateMeshInfo {}

pub struct ReadyRenderableInfo {
    pub engine_id:      u64,
    pub render_mesh_id: RenderId,
}

pub struct CameraStateInfo {
    pub view_matrix:        Float4x4,
    pub perspective_matrix: Float4x4
}

pub struct MaterialInstanceInfo {
    pub ambient_map:   RenderId,
    pub ambient_color: Float3,

    // Some engine-id so that we can tell the engine the mesh has uploaded
    pub engine_id:     u64,
}

pub struct TextureInfo {
    pub name:    String,        //todo: perhaps use a 128bit asset id
    pub format:  super::texture::TextureFormat,
    pub flags:   super::texture::TextureFlags,
    pub sampler: super::texture::SamplerType,
    pub width:   u32,
    pub height:  u32,
    pub depth:   u32,
    pub pixels:  *const u8,    //todo: can we pass ownership of the memory somehow?

    // Some engine-id so that we can tell the engine the mesh has uploaded
    pub engine_id: u64,
}

pub enum RenderCommand{
    // Engine -> Renderer Commands
    //

    // Camera-related commands
    UpdateCamera(CameraStateInfo),

    // Mesh-related commands
    CreateMesh(CreateMeshInfo),
    DestroyMesh,
    HideMesh,
    ShowMesh,

    // Texture-related commands
    CreateTexture(TextureInfo),
    DestroyTexture,

    // Material-related commands
    CreateMaterial(MaterialInstanceInfo),
    DestroyMaterial,

    // Renderer -> Engine Commands
    //

    ReadyMesh(ReadyRenderableInfo),
    ReadyMaterial(ReadyRenderableInfo),
    ReadyTexture(ReadyRenderableInfo),
}

pub struct RenderCommandBuffer{
    pub commands: VecDeque<RenderCommand>,
}

impl Default for RenderCommandBuffer{
    fn default() -> Self{
        Self{
            commands: VecDeque::<RenderCommand>::new(),
        }
    }
}

impl RenderCommandBuffer {
    pub fn add_command(&mut self, cmd: RenderCommand) {
        self.commands.push_back(cmd);
    }
}

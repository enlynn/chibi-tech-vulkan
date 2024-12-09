use std::collections::VecDeque;

use crate::math::float4x4::*;
use super::mesh::Vertex;

pub struct CreateMeshInfo {
    pub vertices:     *const Vertex,
    pub vertex_count: usize,

    pub indices:      *const u32,
    pub index_count:  usize,

    //todo: other mesh properties
    pub transform:    Float4x4,

    // Some engine-id so that we can tell the engine the mesh has uploaded
    pub engine_id:    u64,
}

// Required for sending a *const Vertex
unsafe impl Send for CreateMeshInfo {}

pub struct ReadyMeshInfo {
    pub engine_id:      u64,
    pub render_mesh_id: u64,
}

pub struct CameraStateInfo {
    pub view_matrix:        Float4x4,
    pub perspective_matrix: Float4x4
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
    CreateTexture,
    DestroyTexture,

    // Material-related commands
    CreateMaterial,
    DestroyMaterial,

    // Renderer -> Engine Commands
    //

    ReadyMesh(ReadyMeshInfo),
}

pub struct RenderCommandBuffer{
    pub(crate) commands: VecDeque<RenderCommand>,
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

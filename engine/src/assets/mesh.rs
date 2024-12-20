use common::util::id;

use super::material::*;
use super::AssetLoadState;

pub struct MeshId(id::Id);
pub const INVALID_MESH_ID: MeshId = MeshId(id::INVALID_ID);

pub struct MeshCreateInfo{}

pub struct MeshData {
    name:     String,
    state:    AssetLoadState,
    material: MaterialId,
}

const MAX_MESHES: usize = 1000;
pub struct AssetMeshSystem {

}

impl MeshId {
    pub fn as_id(&self) -> id::Id {
        return self.0;
    }
}

use common::util::id;
use common::math::float3::*;

use super::AssetLoadState;
use super::texture::*;


pub struct MaterialId(id::Id);
pub const INVALID_MATERIAL_ID: MaterialId = MaterialId(id::INVALID_ID);

impl MaterialId {
    pub fn as_id(&self) -> id::Id {
        return self.0;
    }
}

pub struct MaterialCreateInfo{}

pub struct MaterialData {
    name:          String,
    state:         AssetLoadState,
    ambient_map:   TextureId,
    ambient_color: Float3,
}

const MAX_TEXTURES: usize = 1000;
pub struct AssetMaterialSystem {

}

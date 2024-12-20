use common::util::id::*;
use crate::renderer::command_buffer::*;

use super::AssetLoadState;

pub struct TextureId(Id);
pub const INVALID_TEXTURE_ID: TextureId = TextureId(INVALID_ID);

pub struct TextureCreateInfo{}

#[derive(Clone, Copy)]
pub struct TextureData {
    //name:     String, don't store the name here! causes array init problems
    state:    AssetLoadState,
}

const MAX_TEXTURES: usize = 1000;
pub struct AssetTextureSystem {
    textures:           [TextureData; MAX_TEXTURES],
    id_gen:             IdSystem,

    to_upload_commands: [TextureData; MAX_TEXTURES], //todo: this should be RenderTextureUpload
    command_count:      usize,
}

impl TextureId {
    #[inline(always)]
    pub fn as_id(&self) -> Id {
        return self.0;
    }

    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        return self.0 != INVALID_ID;
    }
}

impl Default for TextureData {
    fn default() -> Self {
        Self{
            //name:  String::default(),
            state: AssetLoadState::Unloaded,
        }
    }
}

impl AssetTextureSystem {
    pub fn new() -> Self {
        Self{
            textures:           [TextureData::default(); MAX_TEXTURES],
            id_gen:             IdSystem::new(MAX_TEXTURES),
            to_upload_commands: [todo!(); MAX_TEXTURES],
            command_count:      0,
        }
    }

    pub fn upload_pending_textures(&self, command_list: &RenderCommandBuffer) {
        todo!()
    }

    pub fn on_render_texture_loaded(&self, asset_id: TextureId, render_id: RenderId) {
        todo!()
    }
}

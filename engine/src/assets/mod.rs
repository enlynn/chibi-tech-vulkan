pub mod material;
pub mod mesh;
pub mod texture;

use assetlib::mesh::*;
use assetlib::material::*;
use assetlib::texture::*;

use mesh::*;
use material::*;
use texture::*;

use std::path::PathBuf;

#[derive(Clone, Copy)]
pub enum AssetLoadState {
    Unloaded,
    Loading,
    Done,
}

pub enum AssetId {
    Mesh(MeshId),
    Texture,
    Material,
    Rml,
}

pub enum AssetType {
    Mesh,
    Material,
    Texture,
    Rml,
}

pub enum AssetCreateInfo {
    Mesh(MeshCreateInfo),
    Material,
    Texture,
    Rml,
}

pub struct AssetSystem {
    engine_path: PathBuf,
    game_path:   PathBuf,
}

impl AssetSystem {

    // loads an asset from the asset table - must be a known asset
    // todo: figure out how to uniquely identify an asset
    pub fn load_asset(&self, asset_type: AssetType, create_info: AssetCreateInfo) -> Option<AssetId> {
        todo!()
    }

    // unloads and asset from memory
    pub fn unload_asset(&self, asset_id: AssetId) -> Option<AssetId> {
        todo!()
    }

    // permanently destroys an asset (deletes imported data on disc)
    pub fn destroy_asset(&self, asset_id: AssetId) -> Option<AssetId> {
        todo!()
    }

    pub fn create_asset(&self, asset_type: AssetType, create_info: AssetCreateInfo) -> Option<AssetId> {
        todo!()
    }

    // only valid with the import feature!
    pub fn import_asset(&self) -> Option<AssetId> {
        todo!()
    }
}

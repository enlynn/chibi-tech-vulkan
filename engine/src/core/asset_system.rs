
/*

Asset Systems

// Let's use a modified version of Godot's approach


Godot Locations for user:// -
Windows: %APPDATA%\Godot\app_userdata\[project_name]
macOS: ~/Library/Application Support/Godot/app_userdata/[project_name]
Linux: ~/.local/share/godot/app_userdata/[project_name]

Godot complies with the XDG Base Directory Specification on Linux/BSD.
You can override the XDG_DATA_HOME, XDG_CONFIG_HOME and XDG_CACHE_HOME environment variables to change the editor and project data paths.

Editor data

Windows: %APPDATA%\Godot\
macOS: ~/Library/Application Support/Godot/
Linux: ~/.local/share/godot/

Editor settings

Windows: %APPDATA%\Godot\
macOS: ~/Library/Application Support/Godot/
Linux: ~/.config/godot/

Cache

Windows: %TEMP%\Godot\
macOS: ~/Library/Caches/Godot/
Linux: ~/.cache/godot/

*/

use std::path::PathBuf;
use std::rc::Rc;

pub enum AssetDrive {
    Res,   // Resources local to the project path
    Usr,   // Resources in the appdata directory
    Priv,  // Resources local to the engine path
}

enum FileType {
    File,
    Directory,
}

pub struct File {
    file_type:     FileType,
    size:          usize,
    name:          String,
    relative_path: String,
    absolute_path: String,
    child_files:   Vec<File>,

    borrows:       Vec<AssetId>, // assets that currently rely on this file

    //inner: os::File,
}

pub struct FileWatcher {
    file: File,
    //todo:
    //inner: os::FileWatcher
}

pub struct FileDrive {
    root_path: String, // not sure what this should actually be
    root_file: File,   // Base of the directory

    watcher:   FileWatcher,
}

pub type AssetId = u64;
pub type FileId  = u64;


pub enum AssetType {
    Unknown,

    ShaderFile,
    ShaderBinary,

    MeshGltf,  // external, unimported asset
    MeshChibi, // internal, converted asset

    Material,
    Texture,

    // I'm sure there will be many more.
}

pub struct Asset {
    asset_type:   AssetType,
    id:           AssetId,
    file_id:      FileId,   // Backing File for the Asset
    meta_file_id: FileId,   // Backing File for the Asset Metadata
}

//
pub struct AssetSystem {
    // Virtual File System
    resource_drive: Rc<FileDrive>,
    user_drive:     Rc<FileDrive>,
    priv_drive:     Rc<FileDrive>,


}

impl AssetSystem {
    pub fn new() {

    }

    pub fn get_root_dir(drive: AssetDrive) -> PathBuf {
        match drive {
            AssetDrive::Res  => todo!(),
            AssetDrive::Usr  => todo!(),
            AssetDrive::Priv => {
                let mut root_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
                root_dir = root_dir.join("assets");

                println!("Root Asset Directory: {:?}", root_dir);

                return root_dir;
            },
        }
    }

    // experimental API
    pub fn load_asset_by_name(drive: AssetDrive, name: &str)  {}
    pub fn load_asset_by_path(drive: AssetDrive, path: &str)  {}
    pub fn load_asset_by_id(drive: AssetDrive,   id: AssetId) {}
}

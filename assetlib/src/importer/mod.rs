pub mod obj;

use image::{ImageReader, DynamicImage};
use std::path::PathBuf;

pub fn load_texture_file(texture_path: &PathBuf) -> Option<DynamicImage> {
    return if let Ok(file) = ImageReader::open(texture_path) {
        if let Ok(decoded_image) = file.decode() {
            Some(decoded_image)
        } else {
            None
        }
    } else {
        None
    };
}

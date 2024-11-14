
use crate::window::NativeSurface;


pub struct Features {}

pub struct CreateInfo {
    pub features: Features,
    pub surface: NativeSurface,
}

pub struct Device {}

impl Device {
    pub fn new(_create_info: CreateInfo) -> Device {
        return Device{};
    }
}

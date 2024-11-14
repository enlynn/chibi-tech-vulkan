use std::os::raw;

mod glfw;
use glfw as imp;

//
// Windowing System
//

pub struct WindowSystem {
    inner: imp::WindowSystem,
}

impl WindowSystem {
    pub fn new() -> WindowSystem {
        return WindowSystem{
            inner: imp::WindowSystem::new(),
        };
    }

    pub fn create_window(&self, title: &str, width: i32, height: i32) -> Window {
        return Window {
            inner: self.inner.create_window(title, width, height)
        };
    }

    pub fn pump_window_message(&self) -> bool {
        return self.inner.pump_window_message();
    }
}

//
// Window Surface Wrapper
//

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
pub struct X11Surface
{
    pub(crate) window:     raw::c_ulong,
    pub(crate) display:    *mut raw::c_void,
}

#[cfg(target_os = "linux")]
#[derive(Clone, Copy)]
pub struct WaylandSurface
{
    pub(crate) surface:    *mut raw::c_void,
    pub(crate) display:    *mut raw::c_void,
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy)]
pub struct Win32Surface
{
    pub(crate) surface:    *mut raw::c_void,
    pub(crate) display:    *mut raw::c_void,
}

#[derive(Clone, Copy)]
pub enum NativeSurface
{
    #[cfg(target_os = "linux")]
    X11(X11Surface),
    #[cfg(target_os = "linux")]
    Wayland(WaylandSurface),
    #[cfg(target_os = "windows")]
    Win32(Win32Surface),
}

//
// Window Wrapper
//

pub struct Window {
    inner: imp::Window,
}

impl Window {
    pub fn should_window_close(&self) -> bool {
        return self.inner.should_window_close();
    }

    pub fn get_native_surface(&self) -> NativeSurface {
        return self.inner.get_native_surface();
    }
}

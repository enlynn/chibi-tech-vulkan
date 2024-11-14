use std::{ffi::CString, os::raw};

mod api;

use api::*;

pub struct WindowSystem
{
    //nothing for now
}

impl WindowSystem {
    pub fn new() -> WindowSystem {
        unsafe {
            glfwInit();
            // Vulkan requires us to set NO_API
            glfwWindowHint(GLFW_CLIENT_API as i32, GLFW_NO_API as i32);
        };
        return WindowSystem {};
    }

    pub fn create_window(&self, title: &str, width: i32, height: i32) -> Window {
        let null_terminated_title = CString::new(title).expect("Failed to convert title to CString.");

        let glfw_window = unsafe {
            glfwCreateWindow(width, height,
                null_terminated_title.as_ptr()  as *const std::os::raw::c_char,
                std::ptr::null::<GLFWmonitor>() as *mut GLFWmonitor,
                std::ptr::null::<GLFWwindow>()  as *mut GLFWwindow)
        };

        // todo: setup window callbacks

        return Window::new(glfw_window);
    }

    pub fn pump_window_message(&self) -> bool {
        unsafe {
            glfwPollEvents();
        }

        return true;
    }
}

impl Drop for WindowSystem {
    fn drop(&mut self) {
        unsafe { glfwTerminate(); }
    }
}

//
// GLFW Window
//

extern "C" {
    // XLib bindings
    //
    #[cfg(target_os = "linux")]
    pub fn glfwGetX11Display() -> *mut raw::c_void; // returns Display*

    #[cfg(target_os = "linux")]
    pub fn glfwGetX11Window (window: *mut GLFWwindow) -> raw::c_ulong; // Returns Window

    // Wayland bindings
    //

    // Win32 Bindings
    //
}

enum SupportedPlatform {
    Unknown,
    Xlib,
    Wayland,
    Win32,
}

pub struct Window {
    handle: *mut GLFWwindow,
    platform: SupportedPlatform,
}

impl Window {
    pub fn new(window: *mut GLFWwindow) -> Window {
        let platform = if cfg!(unix) {
            // todo: check wayland

            let display = unsafe { glfwGetX11Display() };
            if display == std::ptr::null::<raw::c_void>() as *mut raw::c_void {
                // Failed to fetch the display
                panic!("Failed to determine windowing platform.");
            }

            SupportedPlatform::Xlib
        } else if cfg!(windows) {
            SupportedPlatform::Win32
        } else {
            panic!("Unsupported platform!");
            //SupportedPlatform::Unknown
        };

        return Window { handle: window, platform };
    }

    pub fn should_window_close(&self) -> bool {
        return unsafe { glfwWindowShouldClose(self.handle) == GLFW_TRUE as i32 };
    }

    pub fn get_native_surface(&self) -> super::NativeSurface {
        use super::X11Surface;

        let native_surface = match self.platform {
            SupportedPlatform::Unknown => { panic!("Unsupported platform!"); },
            SupportedPlatform::Xlib    => {
                let display = unsafe { glfwGetX11Display() };
                let surface = unsafe { glfwGetX11Window(self.handle) };

                super::NativeSurface::X11(X11Surface{
                    window: surface,
                    display,
                })
            },
            SupportedPlatform::Wayland => todo!(),
            SupportedPlatform::Win32   => todo!(),
        };

        return native_surface;
    }
}

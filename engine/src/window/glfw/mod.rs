use std::{ffi::CString, os::raw};

use crate::window::WaylandSurface;

use vendor::glfw::*;

pub struct WindowSystem
{
    //nothing for now
}

//typedef void(* GLFWerrorfun) (int error_code, const char *description)
unsafe extern "C" fn glfw_error_callback(data: raw::c_int, description: *const std::os::raw::c_char)
{
    let desc = crate::util::ffi::cstr_ptr_to_str!(description);
    println!("GLFW Error :: Code({}) :: {:?}", data, desc);
}

impl WindowSystem {
    pub fn new() -> WindowSystem {
        unsafe {
            glfwSetErrorCallback(Some(glfw_error_callback));
            glfwInit();
            // Vulkan requires us to set NO_API
            glfwWindowHint(GLFW_CLIENT_API as i32, GLFW_NO_API as i32);
        };
        return WindowSystem {};
    }

    pub fn create_window(&self, title: &str, width: i32, height: i32) -> Window {
        let null_terminated_title = CString::new(title).expect("Failed to convert title to CString.");

        let glfw_window = unsafe {
            glfwWindowHint(GLFW_CONTEXT_VERSION_MAJOR as i32, 3);
            glfwWindowHint(GLFW_CONTEXT_VERSION_MINOR as i32, 4);

            glfwCreateWindow(width, height,
                null_terminated_title.as_ptr()  as *const std::os::raw::c_char,
                std::ptr::null::<GLFWmonitor>() as *mut GLFWmonitor,
                std::ptr::null::<GLFWwindow>()  as *mut GLFWwindow)
        };

        unsafe { glfwShowWindow(glfw_window) };

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

    //struct wl_display * 	glfwGetWaylandDisplay (void)
 	//   Returns the struct wl_display* used by GLFW.
    #[cfg(target_os = "linux")]
    pub fn glfwGetWaylandDisplay() -> *mut raw::c_void;

    //struct wl_output * 	glfwGetWaylandMonitor (GLFWmonitor *monitor)
 	//   Returns the struct wl_output* of the specified monitor.
    #[cfg(target_os = "linux")]
    pub fn glfwGetWaylandMonitor(window: *mut GLFWwindow) -> *mut raw::c_void;

    //struct wl_surface * 	glfwGetWaylandWindow (GLFWwindow *window)
    //   Returns the main struct wl_surface* of the specified window.
    pub fn glfwGetWaylandWindow(window: *mut GLFWwindow) -> *mut raw::c_void;

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
            if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
                match session_type.as_str() {
                    "x11"     => { println!("Platform is xlib.");    SupportedPlatform::Xlib    },
                    "wayland" => { println!("Platform is wayland."); SupportedPlatform::Wayland },
                    _         => panic!("Unsupported window manager"),
                }
            } else {
                panic!("Unsupported window manager");
            }
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
            SupportedPlatform::Wayland => {
                super::NativeSurface::Wayland(WaylandSurface{
                    surface: unsafe { glfwGetWaylandWindow(self.handle) },
                    display: unsafe { glfwGetWaylandDisplay()           },
                })
            },
            SupportedPlatform::Win32   => todo!(),
        };

        return native_surface;
    }

    pub fn get_framebuffer_size(&self) -> (u32, u32) {
        let mut width  = 0;
        let mut height = 0;

        unsafe { glfwGetFramebufferSize(self.handle, &mut width, &mut height); };

        (width as u32, height as u32)
    }
}

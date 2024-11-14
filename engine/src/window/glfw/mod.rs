
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
        let glfw_window = unsafe {
            glfwCreateWindow(width, height,
                title.as_ptr() as *const std::os::raw::c_char,
                std::ptr::null::<GLFWmonitor>() as *mut GLFWmonitor,
                std::ptr::null::<GLFWwindow>()  as *mut GLFWwindow)
        };

        // todo: setup window callbacks

        return Window{
            handle: glfw_window,
        };
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

pub struct Window {
    handle: *mut GLFWwindow,
}

impl Window {
    pub fn should_window_close(&self) -> bool {
        return unsafe { glfwWindowShouldClose(self.handle) == GLFW_TRUE as i32 };
    }
}

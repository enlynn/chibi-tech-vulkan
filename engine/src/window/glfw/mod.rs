//mod glfw_imgui;

use std::ffi::c_void;
use std::ptr;
use std::{ffi::CString, os::raw};

use crate::window::WaylandSurface;
use crate::util::ffi::call;

use vendor::glfw::*;
use vendor::imgui::*;

use super::KeyModMask;

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

            // imgui setup
            igCreateContext(ptr::null_mut());

            let io = { &mut *igGetIO() }; // gets a mutable reference
            io.MouseDrawCursor = true; // enable software cursor. get a lot of rendering lag without it.
            io.ConfigFlags |= ImGuiConfigFlags_NavEnableKeyboard as i32;   // Enable Keyboard Controls
            //io.ConfigFlags |= ImGuiConfigFlags_NavEnableGamepad  as i32;   // Enable Gamepad Controls
            //todo: figure out
            //io.ConfigFlags |= ImGuiConfigFlags_DockingEnable as i32;       // Enable Docking
            //io.ConfigFlags |= ImGuiConfigFlags_ViewportsEnable as i32;     // Enable Multi-Viewport / Platform Windows

            //note: i have a style somewhere in an old project.
            igStyleColorsDark(ptr::null_mut());

            // When viewports are enabled we tweak WindowRounding/WindowBg so platform windows can look identical to regular ones.
            let style = { &mut *igGetStyle() };
            if (io.ConfigFlags & ImGuiConfigFlags_ViewportsEnable as i32) != 0
            {
                style.WindowRounding = 0.0;
                style.Colors[ImGuiCol_WindowBg as usize].w = 1.0;
            }

        };
        return WindowSystem {};
    }

    pub fn create_window(&self, title: &str, width: i32, height: i32) -> Box<Window> {
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

struct WindowEventSystem {
    listeners: [Vec<super::EventListener>; super::WindowEventType::Count as usize],
}

pub struct Window {
    handle:   *mut GLFWwindow,
    platform: SupportedPlatform,
    events:   WindowEventSystem,
}

unsafe extern "C" fn glfw_key_callback(
    window:   *mut GLFWwindow,
    key:      ::std::os::raw::c_int,
    scancode: ::std::os::raw::c_int,
    action:   ::std::os::raw::c_int,
    mods:     ::std::os::raw::c_int,
) {
    let raw_user_data = glfwGetWindowUserPointer(window);
    let user_window = raw_user_data as *mut Window;

    if user_window != std::ptr::null_mut() {
        let key_event = super::WindowEvent::KeyPress(super::KeyEvent {
            key:   glfw_to_window_key(key),
            state: glfw_to_window_key_action(action),
            mods:  mods as super::KeyModMask,
        });

        (&*user_window).send_event(super::WindowEventType::OnKeyboardKey, key_event);
    }
}

unsafe extern "C" fn glfw_mouse_pos_callback(window: *mut GLFWwindow, xpos: f64, ypos: f64) {
    let raw_user_data = glfwGetWindowUserPointer(window);
    let user_window = raw_user_data as *mut Window;

    if user_window != std::ptr::null_mut() {
        let ev = super::WindowEvent::MouseMove(super::MouseMoveEvent{
            pos_x: xpos,
            pos_y: ypos,
        });

        (&*user_window).send_event(super::WindowEventType::OnMouseMove, ev);
    }
}

unsafe extern "C" fn glfw_mouse_button_callback(
    window: *mut GLFWwindow,
    button: ::std::os::raw::c_int,
    action: ::std::os::raw::c_int,
    mods:   ::std::os::raw::c_int,
) {
    let raw_user_data = glfwGetWindowUserPointer(window);
    let user_window = raw_user_data as *mut Window;

    if user_window != std::ptr::null_mut() {
        let button_ev = super::WindowEvent::MousePress(super::MouseEvent{
            button: glfw_to_mouse_button(button),
            state:  glfw_to_window_key_action(action),
            mods:   mods as super::KeyModMask,
        });

        (&*user_window).send_event(super::WindowEventType::OnMouseButton, button_ev);
    }
}

unsafe extern "C" fn glfw_mouse_scroll_callback(window: *mut GLFWwindow, xoffset: f64, yoffset: f64) {
    let raw_user_data = glfwGetWindowUserPointer(window);
    let user_window = raw_user_data as *mut Window;

    if user_window != std::ptr::null_mut() {
        let ev = super::WindowEvent::MouseScroll(if yoffset > 0.0 { 1 } else if yoffset < 0.0 { -1 } else { 0 });
        (&*user_window).send_event(super::WindowEventType::OnMouseScroll, ev);
    }
}

impl Window {
    pub fn new(window: *mut GLFWwindow) -> Box<Window> {
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

        // Disabling imgui for now...
        //ig_glfw_init(window, true);

        let mut result = Box::new(Window {
            handle: window,
            platform,
            events: WindowEventSystem { listeners: [ Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new() ] },
        });

        // set user data
        let mut user_ptr = &mut *result as *mut Window;
        unsafe { glfwSetWindowUserPointer(result.handle, user_ptr as *mut c_void); };

        // Set Window callbacks
        unsafe {
            glfwSetKeyCallback(window,         Some(glfw_key_callback));
            glfwSetCursorPosCallback(window,   Some(glfw_mouse_pos_callback));
            glfwSetMouseButtonCallback(window, Some(glfw_mouse_button_callback));
            glfwSetScrollCallback(window,      Some(glfw_mouse_scroll_callback));
        }

        return result;
    }

    pub fn register_event(&mut self, event: super::WindowEventType, listener: super::EventListener) {
        assert!(event < super::WindowEventType::Count);
        self.events.listeners[event as usize].push(listener);
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

    pub fn send_event(&self, ev_type: super::WindowEventType, event: super::WindowEvent) {
        for sender in &self.events.listeners[ev_type as usize] {
            sender.send(event);
        }
    }
}

fn glfw_to_window_key(key: i32) -> super::KeyboardKey {
    use super::KeyboardKey;

    let mut result = KeyboardKey::Unknown;

    match key as u32 {
        GLFW_KEY_SPACE         => result = KeyboardKey::Space,
        GLFW_KEY_APOSTROPHE    => result = KeyboardKey::Apostrophe,
        GLFW_KEY_COMMA         => result = KeyboardKey::Comma,
        GLFW_KEY_MINUS         => result = KeyboardKey::Minus,
        GLFW_KEY_PERIOD        => result = KeyboardKey::Period,
        GLFW_KEY_SLASH         => result = KeyboardKey::Slash,
        GLFW_KEY_0             => result = KeyboardKey::Zero,
        GLFW_KEY_1             => result = KeyboardKey::One,
        GLFW_KEY_2             => result = KeyboardKey::Two,
        GLFW_KEY_3             => result = KeyboardKey::Three,
        GLFW_KEY_4             => result = KeyboardKey::Four,
        GLFW_KEY_5             => result = KeyboardKey::Five,
        GLFW_KEY_6             => result = KeyboardKey::Six,
        GLFW_KEY_7             => result = KeyboardKey::Seven,
        GLFW_KEY_8             => result = KeyboardKey::Eight,
        GLFW_KEY_9             => result = KeyboardKey::Nine,
        GLFW_KEY_SEMICOLON     => result = KeyboardKey::Semicolon,
        GLFW_KEY_EQUAL         => result = KeyboardKey::Equal,
        GLFW_KEY_A             => result = KeyboardKey::A,
        GLFW_KEY_B             => result = KeyboardKey::B,
        GLFW_KEY_C             => result = KeyboardKey::C,
        GLFW_KEY_D             => result = KeyboardKey::D,
        GLFW_KEY_E             => result = KeyboardKey::E,
        GLFW_KEY_F             => result = KeyboardKey::F,
        GLFW_KEY_G             => result = KeyboardKey::G,
        GLFW_KEY_H             => result = KeyboardKey::H,
        GLFW_KEY_I             => result = KeyboardKey::I,
        GLFW_KEY_J             => result = KeyboardKey::J,
        GLFW_KEY_K             => result = KeyboardKey::K,
        GLFW_KEY_L             => result = KeyboardKey::L,
        GLFW_KEY_M             => result = KeyboardKey::M,
        GLFW_KEY_N             => result = KeyboardKey::N,
        GLFW_KEY_O             => result = KeyboardKey::O,
        GLFW_KEY_P             => result = KeyboardKey::P,
        GLFW_KEY_Q             => result = KeyboardKey::Q,
        GLFW_KEY_R             => result = KeyboardKey::R,
        GLFW_KEY_S             => result = KeyboardKey::S,
        GLFW_KEY_T             => result = KeyboardKey::T,
        GLFW_KEY_U             => result = KeyboardKey::U,
        GLFW_KEY_V             => result = KeyboardKey::V,
        GLFW_KEY_W             => result = KeyboardKey::W,
        GLFW_KEY_X             => result = KeyboardKey::X,
        GLFW_KEY_Y             => result = KeyboardKey::Y,
        GLFW_KEY_Z             => result = KeyboardKey::Z,
        GLFW_KEY_LEFT_BRACKET  => result = KeyboardKey::LeftBracket,
        GLFW_KEY_BACKSLASH     => result = KeyboardKey::Backslash,
        GLFW_KEY_RIGHT_BRACKET => result = KeyboardKey::RightBracket,
        GLFW_KEY_GRAVE_ACCENT  => result = KeyboardKey::GraveAccent,
        GLFW_KEY_WORLD_1       => result = KeyboardKey::World1,
        GLFW_KEY_WORLD_2       => result = KeyboardKey::World2,
        GLFW_KEY_ESCAPE        => result = KeyboardKey::Escape,
        GLFW_KEY_ENTER         => result = KeyboardKey::Enter,
        GLFW_KEY_TAB           => result = KeyboardKey::Tab,
        GLFW_KEY_BACKSPACE     => result = KeyboardKey::Backspace,
        GLFW_KEY_INSERT        => result = KeyboardKey::Insert,
        GLFW_KEY_DELETE        => result = KeyboardKey::Delete,
        GLFW_KEY_RIGHT         => result = KeyboardKey::Right,
        GLFW_KEY_LEFT          => result = KeyboardKey::Left,
        GLFW_KEY_DOWN          => result = KeyboardKey::Down,
        GLFW_KEY_UP            => result = KeyboardKey::Up,
        GLFW_KEY_PAGE_UP       => result = KeyboardKey::PageUp,
        GLFW_KEY_PAGE_DOWN     => result = KeyboardKey::PageDown,
        GLFW_KEY_HOME          => result = KeyboardKey::Home,
        GLFW_KEY_END           => result = KeyboardKey::End,
        GLFW_KEY_CAPS_LOCK     => result = KeyboardKey::CapsLock,
        GLFW_KEY_SCROLL_LOCK   => result = KeyboardKey::ScollLock,
        GLFW_KEY_NUM_LOCK      => result = KeyboardKey::NumLock,
        GLFW_KEY_PRINT_SCREEN  => result = KeyboardKey::PrintScreen,
        GLFW_KEY_PAUSE         => result = KeyboardKey::Pause,
        GLFW_KEY_F1            => result = KeyboardKey::F1,
        GLFW_KEY_F2            => result = KeyboardKey::F2,
        GLFW_KEY_F3            => result = KeyboardKey::F3,
        GLFW_KEY_F4            => result = KeyboardKey::F4,
        GLFW_KEY_F5            => result = KeyboardKey::F5,
        GLFW_KEY_F6            => result = KeyboardKey::F6,
        GLFW_KEY_F7            => result = KeyboardKey::F7,
        GLFW_KEY_F8            => result = KeyboardKey::F8,
        GLFW_KEY_F9            => result = KeyboardKey::F9,
        GLFW_KEY_F10           => result = KeyboardKey::F10,
        GLFW_KEY_F11           => result = KeyboardKey::F11,
        GLFW_KEY_F12           => result = KeyboardKey::F12,
        GLFW_KEY_F13           => result = KeyboardKey::F13,
        GLFW_KEY_F14           => result = KeyboardKey::F14,
        GLFW_KEY_F15           => result = KeyboardKey::F15,
        GLFW_KEY_F16           => result = KeyboardKey::F16,
        GLFW_KEY_F17           => result = KeyboardKey::F17,
        GLFW_KEY_F18           => result = KeyboardKey::F18,
        GLFW_KEY_F19           => result = KeyboardKey::F19,
        GLFW_KEY_F20           => result = KeyboardKey::F20,
        GLFW_KEY_F21           => result = KeyboardKey::F21,
        GLFW_KEY_F22           => result = KeyboardKey::F22,
        GLFW_KEY_F23           => result = KeyboardKey::F23,
        GLFW_KEY_F24           => result = KeyboardKey::F24,
        GLFW_KEY_F25           => result = KeyboardKey::F25,
        GLFW_KEY_KP_0          => result = KeyboardKey::KP0,
        GLFW_KEY_KP_1          => result = KeyboardKey::KP1,
        GLFW_KEY_KP_2          => result = KeyboardKey::KP2,
        GLFW_KEY_KP_3          => result = KeyboardKey::KP3,
        GLFW_KEY_KP_4          => result = KeyboardKey::KP4,
        GLFW_KEY_KP_5          => result = KeyboardKey::KP5,
        GLFW_KEY_KP_6          => result = KeyboardKey::KP6,
        GLFW_KEY_KP_7          => result = KeyboardKey::KP7,
        GLFW_KEY_KP_8          => result = KeyboardKey::KP8,
        GLFW_KEY_KP_9          => result = KeyboardKey::KP9,
        GLFW_KEY_KP_DECIMAL    => result = KeyboardKey::KPDecimal,
        GLFW_KEY_KP_DIVIDE     => result = KeyboardKey::KPDivide,
        GLFW_KEY_KP_MULTIPLY   => result = KeyboardKey::KPMultiply,
        GLFW_KEY_KP_SUBTRACT   => result = KeyboardKey::KPSubtract,
        GLFW_KEY_KP_ADD        => result = KeyboardKey::KPAdd,
        GLFW_KEY_KP_ENTER      => result = KeyboardKey::KPEnter,
        GLFW_KEY_KP_EQUAL      => result = KeyboardKey::KPEqual,
        GLFW_KEY_LEFT_SHIFT    => result = KeyboardKey::LeftShift,
        GLFW_KEY_LEFT_CONTROL  => result = KeyboardKey::LeftControl,
        GLFW_KEY_LEFT_ALT      => result = KeyboardKey::LeftAlt,
        GLFW_KEY_LEFT_SUPER    => result = KeyboardKey::LeftSuper,
        GLFW_KEY_RIGHT_SHIFT   => result = KeyboardKey::RightShift,
        GLFW_KEY_RIGHT_CONTROL => result = KeyboardKey::RightControl,
        GLFW_KEY_RIGHT_ALT     => result = KeyboardKey::RightAlt,
        GLFW_KEY_RIGHT_SUPER   => result = KeyboardKey::RightSuper,
        GLFW_KEY_MENU          => result = KeyboardKey::Menu,
        GLFW_KEY_LAST          => result = KeyboardKey::Last,
        default                => result = KeyboardKey::Unknown,
    }

    return result;
}

fn glfw_to_window_key_action(action: i32) -> super::KeyState {
    match action as u32 {
        GLFW_RELEASE     => return super::KeyState::Released,
        GLFW_PRESS       => return super::KeyState::Pressed,
        GLFW_REPEAT      => return super::KeyState::Held,
        default          => return super::KeyState::Unknown,
    }
}

fn glfw_to_mouse_button(button: i32) -> super::MouseButton {
    match button as u32 {
        GLFW_MOUSE_BUTTON_1      => return super::MouseButton::ButtonLeft,
        GLFW_MOUSE_BUTTON_2      => return super::MouseButton::ButtonRight,
        GLFW_MOUSE_BUTTON_3      => return super::MouseButton::ButtonMiddle,
        GLFW_MOUSE_BUTTON_4      => return super::MouseButton::Button4,
        GLFW_MOUSE_BUTTON_5      => return super::MouseButton::Button5,
        GLFW_MOUSE_BUTTON_6      => return super::MouseButton::Button6,
        GLFW_MOUSE_BUTTON_7      => return super::MouseButton::Button7,
        GLFW_MOUSE_BUTTON_8      => return super::MouseButton::Button8,
        default                  => return super::MouseButton::Unknown,
    }
}

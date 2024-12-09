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
// Window Events
//

pub type KeyModMask = u16;
pub const KEY_MOD_FLAG_NONE:      u16 = 0x000;
pub const KEY_MOD_FLAG_SHIFT:     u16 = 0x001;
pub const KEY_MOD_FLAG_CONTROL:   u16 = 0x002;
pub const KEY_MOD_FLAG_ALT:       u16 = 0x004;
pub const KEY_MOD_FLAG_SUPER:     u16 = 0x008;
pub const KEY_MOD_FLAG_CAPS_LOCK: u16 = 0x010;
pub const KEY_MOD_FLAG_NUM_LOCK:  u16 = 0x020;

#[derive(PartialOrd, PartialEq)]
pub enum KeyMod {
    None,
    Shift,
    Control,
    Alt,
    Super,
    CapsLock,
    NumLock,
    Count,
}

pub fn is_key_mod_flag_set(mask: KeyModMask, modifier: KeyMod) -> bool {
    assert!(modifier < KeyMod::Count);

    let flag_list: [KeyModMask; KeyMod::Count as usize] =  [
        KEY_MOD_FLAG_NONE,
        KEY_MOD_FLAG_SHIFT,
        KEY_MOD_FLAG_CONTROL,
        KEY_MOD_FLAG_ALT,
        KEY_MOD_FLAG_SUPER,
        KEY_MOD_FLAG_CAPS_LOCK,
        KEY_MOD_FLAG_NUM_LOCK,
    ];

    return (mask & flag_list[modifier as usize]) != 0;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KeyboardKey {
    Unknown,
    Space,
    Apostrophe,   // '
    Comma,        // ,
    Minus,        // -
    Period,       // .
    Slash,        // /
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Semicolon,   // ;
    Equal,       // =
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,    // [
    Backslash,       // \
    RightBracket,   // ]
    GraveAccent,    // `
    World1,         // non-US #1
    World2,         // non-US #2
    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock,
    ScollLock,
    NumLock,
    PrintScreen,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    KP0,
    KP1,
    KP2,
    KP3,
    KP4,
    KP5,
    KP6,
    KP7,
    KP8,
    KP9,
    KPDecimal,
    KPDivide,
    KPMultiply,
    KPSubtract,
    KPAdd,
    KPEnter,
    KPEqual,
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    Menu,
    Last,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MouseButton {
    Unknown      = 0,
    ButtonLeft   = 1,
    ButtonRight  = 2,
    ButtonMiddle = 3,
    Button4      = 4,
    Button5      = 5,
    Button6      = 6,
    Button7      = 7,
    Button8      = 8,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KeyState {
    Unknown,
    Pressed,
    Released,
    Held,
}

#[derive(Debug, Copy, Clone)]
pub struct KeyEvent {
    pub key:   KeyboardKey,
    pub state: KeyState,
    pub mods:  KeyModMask,
}

#[derive(Debug, Copy, Clone)]
pub struct MouseEvent {
    pub button: MouseButton,
    pub state:  KeyState,
    pub mods:   KeyModMask,
}

#[derive(Debug, Copy, Clone)]
pub struct MouseMoveEvent {
    pub pos_x: f64, // position in pixels, oriented to the top left of window
    pub pos_y: f64, // position in pixels, oriented to the top left of window
}

#[derive(Debug, Copy, Clone)]
pub struct WindowResizeEvent {
    pub width:  u32,
    pub height: u32,
}

#[derive(Debug, Copy, Clone)]
pub enum WindowEvent {
    KeyPress(KeyEvent),
    MousePress(MouseEvent),
    MouseMove(MouseMoveEvent),
    MouseScroll(i32),
    Resize(WindowResizeEvent),
}

#[derive(PartialEq, PartialOrd)]
pub enum WindowEventType {
    OnKeyboardKey = 0, // Register for key press, release, and held events
    OnMouseButton = 1, // Register for mouse button press, release, and held events
    OnMouseMove   = 2, // Register for mouse move events
    OnMouseScroll = 3, // Register for mouse scroll events
    OnResize      = 4, // Register for window resize events
    Count         = 5,
}

use std::sync::mpsc;
pub type EventReciever = mpsc::Receiver<WindowEvent>;
pub type EventListener = mpsc::Sender<WindowEvent>;

pub fn make_event_channels() -> (EventListener, EventReciever) {
    let (channel_sender, channel_reciever) = mpsc::channel();
    (channel_sender, channel_reciever)
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
    inner: Box<imp::Window>,
}

impl Window {
    pub fn should_window_close(&self) -> bool {
        return self.inner.should_window_close();
    }

    pub fn get_native_surface(&self) -> NativeSurface {
        return self.inner.get_native_surface();
    }

    pub fn get_framebuffer_size(&self) -> (u32, u32) {
        return self.inner.get_framebuffer_size();
    }

    pub fn register_event(&mut self, ev_type: WindowEventType, listener: EventListener) {
        self.inner.register_event(ev_type, listener);
    }
}

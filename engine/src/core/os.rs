mod unix {
    use std::borrow::Cow;
    use std::ffi::CStr;
    use std::os::raw;

    use crate::util::ffi::*;

    #[cfg(any(target_os = "linux", target_os = "android"))]
    const RTLD_LOCAL: raw::c_int = 0;
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    const RTLD_LOCAL: raw::c_int = 4;

    #[cfg(all(target_os = "android", target_pointer_width = "32"))]
    const RTLD_NOW: raw::c_int = 0;
    #[cfg(any(
        target_os = "linux",
        all(target_os = "android", target_pointer_width = "64"),
        target_os = "macos",
        target_os = "ios"
    ))]
    const RTLD_NOW: raw::c_int = 2;

    extern "C" {
        fn dlopen(filename: *const raw::c_char, flag: raw::c_int) -> *mut raw::c_void;
        fn dlsym(handle: *mut raw::c_void, symbol: *const raw::c_char) -> *mut raw::c_void;
        fn dlclose(handle: *mut raw::c_void) -> raw::c_int;
    }

    pub struct DllLibrary
    {
        lib: *mut raw::c_void,
    }

    impl DllLibrary {
        pub fn load(path: &str) -> Option<DllLibrary> {
            let path_with_nul: Cow<str> = get_nul_terminated_string(path)?;

            let lib = unsafe {
                let path_cstr = CStr::from_bytes_with_nul_unchecked(path_with_nul.as_bytes());
                dlopen(path_cstr.as_ptr(), RTLD_NOW | RTLD_LOCAL)
            };
            if lib.is_null() {
                None
            } else {
                Some(DllLibrary { lib })
            }
        }

        pub fn get_fn<T>(self: &DllLibrary, fn_name: &str) -> Option<T> {
            let name_with_nul: Cow<str> = get_nul_terminated_string(fn_name)?;
            unsafe {
                let name_cstr = CStr::from_bytes_with_nul_unchecked(name_with_nul.as_bytes());
                let ptr = dlsym(self.lib, name_cstr.as_ptr());
                std::mem::transmute_copy(&ptr)
            }
        }
    }

    impl Drop for DllLibrary {
        fn drop(&mut self) {
            unsafe { dlclose(self.lib) };
        }
    }
}

#[cfg(unix)]
pub use unix::*;

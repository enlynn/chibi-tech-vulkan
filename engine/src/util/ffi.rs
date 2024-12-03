use std::borrow::Cow;

/* ======================================================================== */
/* helpful ffi macros and functions                                         */

macro_rules! cstr_stringify {
    ($ident:ident) => {
        #[allow(unused_unsafe)]
        unsafe {
            std::ffi::CStr::from_bytes_with_nul_unchecked(concat!(stringify!($ident), "\0").as_bytes()).as_ptr()
        }
    };
}

macro_rules! str_as_cstr_unchecked {
    ($expr:expr) => {
        #[allow(unused_unsafe)]
        unsafe {
            std::ffi::CStr::from_ptr($expr.as_ptr() as *const i8).as_ptr()
        }
    };
}

macro_rules! char_array_as_cstr {
    ($expr:expr) => {
        #[allow(unused_unsafe)]
        unsafe {
            std::ffi::CStr::from_ptr($expr.as_ptr())
        }
    };
}

macro_rules! byte_array_as_cstr {
    ($expr:expr) => {
        #[allow(unused_unsafe)]
        unsafe {
            std::ffi::CStr::from_bytes_with_nul_unchecked($expr)
        }
    };
}

macro_rules! cstr_ptr_to_str {
    ($expr:expr) => {{
        extern "C" {
            fn strlen(s: *const std::os::raw::c_char) -> usize;
        }

        let val = $expr;
        let len = strlen(val);
        let slice = std::slice::from_raw_parts(val as *const u8, len);
        std::str::from_utf8_unchecked(slice)
    }};
}

macro_rules! call {
    ($call:expr, $($arg:expr),*) => {{
        #[allow(unused_unsafe)]
        unsafe { ($call)($($arg,)*) }
    }};
    ($call:expr) => {{
        #[allow(unused_unsafe)]
        unsafe { ($call)() }
    }};
}

#[inline]
pub(crate) fn get_nul_terminated_string(string: &str) -> Option<Cow<str>>
{
    if let Some(last) = string.as_bytes().last() {
        if *last == 0 {
            Some(Cow::Borrowed(string))
        } else {
            let mut owned = string.to_string();
            owned.push('\0');
            Some(Cow::Owned(owned))
        }
    } else {
        None
    }
}

#[inline]
pub(crate) fn get_nul_terminated_string_always(string: &str) -> Cow<str>
{
    if let Some(last) = string.as_bytes().last() {
        if *last == 0 {
            Cow::Borrowed(string)
        } else {
            let mut owned = string.to_string();
            owned.push('\0');
            Cow::Owned(owned)
        }
    } else {
        Cow::Borrowed("\0")
    }
}

pub(crate) use {byte_array_as_cstr, call, char_array_as_cstr, cstr_ptr_to_str, cstr_stringify, str_as_cstr_unchecked};

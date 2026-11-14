//! Custom controller implementation for user-defined device control.

use crate::common::ControllerFeature;
use crate::sys;
use std::ffi::{CStr, CString};
use std::os::raw::c_void;

/// Custom controller callback trait for implementing custom device control.
pub trait CustomControllerCallback: Send + Sync {
    fn connect(&self) -> bool;
    fn connected(&self) -> bool {
        true
    }
    fn request_uuid(&self) -> Option<String> {
        None
    }
    /// Get the features supported by this controller.
    ///
    /// Returns a combination of `ControllerFeature` flags.
    fn get_features(&self) -> ControllerFeature {
        ControllerFeature::empty()
    }
    fn start_app(&self, _intent: &str) -> bool {
        false
    }
    fn stop_app(&self, _intent: &str) -> bool {
        false
    }
    /// Returns PNG-encoded screenshot data.
    fn screencap(&self) -> Option<Vec<u8>> {
        None
    }
    fn click(&self, _x: i32, _y: i32) -> bool {
        false
    }
    fn swipe(&self, _x1: i32, _y1: i32, _x2: i32, _y2: i32, _duration: i32) -> bool {
        false
    }
    fn touch_down(&self, _contact: i32, _x: i32, _y: i32, _pressure: i32) -> bool {
        false
    }
    fn touch_move(&self, _contact: i32, _x: i32, _y: i32, _pressure: i32) -> bool {
        false
    }
    fn touch_up(&self, _contact: i32) -> bool {
        false
    }
    fn click_key(&self, _keycode: i32) -> bool {
        false
    }
    fn input_text(&self, _text: &str) -> bool {
        false
    }
    fn key_down(&self, _keycode: i32) -> bool {
        false
    }
    fn key_up(&self, _keycode: i32) -> bool {
        false
    }
    fn scroll(&self, _dx: i32, _dy: i32) -> bool {
        false
    }
    fn relative_move(&self, _dx: i32, _dy: i32) -> bool {
        false
    }
    fn shell(&self, _cmd: &str, _timeout: i64) -> Option<String> {
        None
    }
    /// Set the controller to inactive state.
    ///
    /// For Win32 controllers, this restores window position (removes topmost) and unblocks user input.
    /// For other controllers, this is a no-op that always succeeds.
    fn inactive(&self) -> bool {
        true
    }
    /// Get custom controller extra info as a JSON string.
    ///
    /// The returned JSON will be merged with base controller info.
    /// Default implementation returns empty JSON object.
    fn get_info(&self) -> String {
        "{}".to_string()
    }
}

type BoxedCallback = Box<dyn CustomControllerCallback>;

fn guard<T>(callback: &str, fallback: T, f: impl FnOnce() -> T) -> T {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).unwrap_or_else(|_| {
        eprintln!(
            "MaaFramework Rust Binding: Panic caught in custom controller {callback} callback"
        );
        fallback
    })
}

// FFI trampolines
unsafe extern "C" fn connect_trampoline(trans_arg: *mut c_void) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("connect", 0, || cb.connect() as sys::MaaBool)
}

unsafe extern "C" fn connected_trampoline(trans_arg: *mut c_void) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("connected", 0, || cb.connected() as sys::MaaBool)
}

unsafe extern "C" fn request_uuid_trampoline(
    trans_arg: *mut c_void,
    buffer: *mut sys::MaaStringBuffer,
) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("request_uuid", 0, || {
        if let Some(uuid) = cb.request_uuid() {
            if let Ok(c_str) = std::ffi::CString::new(uuid) {
                unsafe {
                    sys::MaaStringBufferSetEx(
                        buffer,
                        c_str.as_ptr(),
                        c_str.as_bytes().len() as u64,
                    );
                }
                return 1;
            }
        }
        0
    })
}

unsafe extern "C" fn get_features_trampoline(trans_arg: *mut c_void) -> sys::MaaControllerFeature {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("get_features", 0, || cb.get_features().bits())
}

unsafe extern "C" fn start_app_trampoline(
    intent: *const std::os::raw::c_char,
    trans_arg: *mut c_void,
) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    let intent_str = if !intent.is_null() {
        unsafe { CStr::from_ptr(intent).to_string_lossy() }
    } else {
        std::borrow::Cow::Borrowed("")
    };
    guard("start_app", 0, || cb.start_app(&intent_str) as sys::MaaBool)
}

unsafe extern "C" fn stop_app_trampoline(
    intent: *const std::os::raw::c_char,
    trans_arg: *mut c_void,
) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    let intent_str = if !intent.is_null() {
        unsafe { CStr::from_ptr(intent).to_string_lossy() }
    } else {
        std::borrow::Cow::Borrowed("")
    };
    guard("stop_app", 0, || cb.stop_app(&intent_str) as sys::MaaBool)
}

unsafe extern "C" fn screencap_trampoline(
    trans_arg: *mut c_void,
    buffer: *mut sys::MaaImageBuffer,
) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("screencap", 0, || match cb.screencap() {
        Some(data) => unsafe {
            sys::MaaImageBufferSetEncoded(buffer, data.as_ptr() as *mut u8, data.len() as u64)
        },
        None => 0,
    })
}

unsafe extern "C" fn click_trampoline(x: i32, y: i32, trans_arg: *mut c_void) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("click", 0, || cb.click(x, y) as sys::MaaBool)
}

unsafe extern "C" fn swipe_trampoline(
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    duration: i32,
    trans_arg: *mut c_void,
) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("swipe", 0, || {
        cb.swipe(x1, y1, x2, y2, duration) as sys::MaaBool
    })
}

unsafe extern "C" fn touch_down_trampoline(
    contact: i32,
    x: i32,
    y: i32,
    pressure: i32,
    trans_arg: *mut c_void,
) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("touch_down", 0, || {
        cb.touch_down(contact, x, y, pressure) as sys::MaaBool
    })
}

unsafe extern "C" fn touch_move_trampoline(
    contact: i32,
    x: i32,
    y: i32,
    pressure: i32,
    trans_arg: *mut c_void,
) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("touch_move", 0, || {
        cb.touch_move(contact, x, y, pressure) as sys::MaaBool
    })
}

unsafe extern "C" fn touch_up_trampoline(contact: i32, trans_arg: *mut c_void) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("touch_up", 0, || cb.touch_up(contact) as sys::MaaBool)
}

unsafe extern "C" fn click_key_trampoline(keycode: i32, trans_arg: *mut c_void) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("click_key", 0, || cb.click_key(keycode) as sys::MaaBool)
}

unsafe extern "C" fn input_text_trampoline(
    text: *const std::os::raw::c_char,
    trans_arg: *mut c_void,
) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    let text_str = if !text.is_null() {
        unsafe { CStr::from_ptr(text).to_string_lossy() }
    } else {
        std::borrow::Cow::Borrowed("")
    };
    guard("input_text", 0, || cb.input_text(&text_str) as sys::MaaBool)
}

unsafe extern "C" fn key_down_trampoline(keycode: i32, trans_arg: *mut c_void) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("key_down", 0, || cb.key_down(keycode) as sys::MaaBool)
}

unsafe extern "C" fn key_up_trampoline(keycode: i32, trans_arg: *mut c_void) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("key_up", 0, || cb.key_up(keycode) as sys::MaaBool)
}

unsafe extern "C" fn scroll_trampoline(dx: i32, dy: i32, trans_arg: *mut c_void) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("scroll", 0, || cb.scroll(dx, dy) as sys::MaaBool)
}

unsafe extern "C" fn relative_move_trampoline(
    dx: i32,
    dy: i32,
    trans_arg: *mut c_void,
) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("relative_move", 0, || {
        cb.relative_move(dx, dy) as sys::MaaBool
    })
}

unsafe extern "C" fn shell_trampoline(
    cmd: *const std::os::raw::c_char,
    timeout: i64,
    trans_arg: *mut c_void,
    buffer: *mut sys::MaaStringBuffer,
) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    let cmd_str = if !cmd.is_null() {
        unsafe { CStr::from_ptr(cmd).to_string_lossy() }
    } else {
        std::borrow::Cow::Borrowed("")
    };

    guard("shell", 0, || {
        if let Some(output) = cb.shell(&cmd_str, timeout) {
            if let Ok(c_str) = CString::new(output) {
                unsafe {
                    sys::MaaStringBufferSetEx(
                        buffer,
                        c_str.as_ptr(),
                        c_str.as_bytes().len() as u64,
                    );
                }
                return 1;
            }
        }
        0
    })
}

unsafe extern "C" fn inactive_trampoline(trans_arg: *mut c_void) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("inactive", 0, || cb.inactive() as sys::MaaBool)
}

unsafe extern "C" fn get_info_trampoline(
    trans_arg: *mut c_void,
    buffer: *mut sys::MaaStringBuffer,
) -> sys::MaaBool {
    let cb = unsafe { &*(trans_arg as *const BoxedCallback) };
    guard("get_info", 0, || {
        if let Ok(c_str) = std::ffi::CString::new(cb.get_info()) {
            unsafe {
                sys::MaaStringBufferSetEx(buffer, c_str.as_ptr(), c_str.as_bytes().len() as u64);
            }
            return 1;
        }
        0
    })
}

pub fn create_custom_controller_callbacks() -> sys::MaaCustomControllerCallbacks {
    sys::MaaCustomControllerCallbacks {
        connect: Some(connect_trampoline),
        connected: Some(connected_trampoline),
        request_uuid: Some(request_uuid_trampoline),
        get_features: Some(get_features_trampoline),
        start_app: Some(start_app_trampoline),
        stop_app: Some(stop_app_trampoline),
        screencap: Some(screencap_trampoline),
        click: Some(click_trampoline),
        swipe: Some(swipe_trampoline),
        touch_down: Some(touch_down_trampoline),
        touch_move: Some(touch_move_trampoline),
        touch_up: Some(touch_up_trampoline),
        click_key: Some(click_key_trampoline),
        input_text: Some(input_text_trampoline),
        key_down: Some(key_down_trampoline),
        key_up: Some(key_up_trampoline),
        scroll: Some(scroll_trampoline),
        relative_move: Some(relative_move_trampoline),
        shell: Some(shell_trampoline),
        inactive: Some(inactive_trampoline),
        get_info: Some(get_info_trampoline),
    }
}

static CALLBACKS: std::sync::OnceLock<sys::MaaCustomControllerCallbacks> =
    std::sync::OnceLock::new();

pub fn get_callbacks() -> &'static sys::MaaCustomControllerCallbacks {
    CALLBACKS.get_or_init(create_custom_controller_callbacks)
}

//! Internal callback infrastructure for FFI event handling.

use crate::sys;
use std::ffi::CStr;
use std::os::raw::c_void;

pub type EventCallbackFn = Box<dyn Fn(&str, &str) + Send + Sync>;

unsafe extern "C" fn event_callback_trampoline(
    _handle: *mut c_void,
    msg: *const std::os::raw::c_char,
    details: *const std::os::raw::c_char,
    trans_arg: *mut c_void,
) {
    if trans_arg.is_null() {
        return;
    }
    let callback = &*(trans_arg as *const EventCallbackFn);

    let msg_str = if !msg.is_null() {
        CStr::from_ptr(msg).to_string_lossy()
    } else {
        std::borrow::Cow::Borrowed("")
    };

    let details_str = if !details.is_null() {
        CStr::from_ptr(details).to_string_lossy()
    } else {
        std::borrow::Cow::Borrowed("")
    };

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        callback(&msg_str, &details_str);
    }));

    if let Err(_) = result {
        eprintln!("MaaFramework Rust Binding: Panic caught in event callback");
    }
}

pub struct EventCallback {
    _ptr: *mut c_void,
}

impl EventCallback {
    pub fn new(
        cb: impl Fn(&str, &str) + Send + Sync + 'static,
    ) -> (sys::MaaEventCallback, *mut c_void) {
        let boxed: EventCallbackFn = Box::new(cb);
        let ptr = Box::into_raw(Box::new(boxed)) as *mut c_void;
        (Some(event_callback_trampoline), ptr)
    }

    pub unsafe fn drop_callback(ptr: *mut c_void) {
        if !ptr.is_null() {
            let _ = Box::from_raw(ptr as *mut EventCallbackFn);
        }
    }

    pub fn new_sink(
        handle: crate::common::MaaId,
        sink: Box<dyn crate::event_sink::EventSink>,
    ) -> (sys::MaaEventCallback, *mut c_void) {
        let wrapper = Box::new(EventSinkWrapper { handle, sink });
        let ptr = Box::into_raw(wrapper) as *mut c_void;
        (Some(event_sink_trampoline), ptr)
    }

    pub unsafe fn drop_sink(ptr: *mut c_void) {
        if !ptr.is_null() {
            let _ = Box::from_raw(ptr as *mut EventSinkWrapper);
        }
    }
}

/// Wrapper to hold the event sink and its associated instance handle.
///
/// This struct is boxed and converted to a raw pointer to be passed as the `callback_arg`
/// to the C API. It ensures the `handle` is available when the callback is invoked.
struct EventSinkWrapper {
    handle: crate::common::MaaId,
    sink: Box<dyn crate::event_sink::EventSink>,
}

/// Trampoline function for safe FFI callback dispatch.
///
/// This function acts as the bridge between the C callback interface and the Rust `EventSink` trait.
/// It:
/// 1. Casts the `callback_arg` back to `EventSinkWrapper`.
/// 2. Converts C-style message strings to Rust `Cow<str>`.
/// 3. Parses the message and details into a typed `MaaEvent`.
/// 4. Dispatches the event to the user's sink implementation with the correct handle.
unsafe extern "C" fn event_sink_trampoline(
    _handle: *mut c_void,
    _msg: *const std::os::raw::c_char,
    _detail: *const std::os::raw::c_char,
    callback_arg: *mut c_void,
) {
    if callback_arg.is_null() {
        return;
    }
    let wrapper = &*(callback_arg as *const EventSinkWrapper);

    let msg_str = if !_msg.is_null() {
        CStr::from_ptr(_msg).to_string_lossy()
    } else {
        std::borrow::Cow::Borrowed("")
    };

    let detail_str = if !_detail.is_null() {
        CStr::from_ptr(_detail).to_string_lossy()
    } else {
        std::borrow::Cow::Borrowed("")
    };

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let event = crate::notification::MaaEvent::from_json(&msg_str, &detail_str);
        wrapper.sink.on_event(wrapper.handle, &event);
    }));

    if let Err(_) = result {
        eprintln!("MaaFramework Rust Binding: Panic caught in event sink callback");
    }
}

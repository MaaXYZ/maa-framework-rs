#![allow(dead_code)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use crate::{msg::MaaMsg, CallbackHandler};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[macro_export]
macro_rules! maa_bool {
    ($v:expr) => {
        $v != 0
    };

    ($v:expr, $e:ident) => {
        if $v != 0 {
            Ok(())
        } else {
            Err(Error::$e)
        }
    };

    ($v:expr, $e:ident, $c:expr) => {
        if $v != 0 {
            Ok(())
        } else {
            Err(Error::$e($c))
        }
    };
}

#[macro_export]
macro_rules! string {
    ($string_view:expr) => {
        unsafe {
            std::ffi::CStr::from_ptr($string_view)
                .to_str()
                .unwrap()
                .to_string()
        }
    };
}

#[macro_export]
macro_rules! string_view {
    ($string:expr) => {
        $string.as_ptr() as *const std::os::raw::c_char
    };
}

pub(crate) unsafe extern "C" fn callback_handler<T: CallbackHandler>(
    msg: *const std::os::raw::c_char,
    details_json: *const std::os::raw::c_char,
    user_data: *mut std::os::raw::c_void,
) {
    let msg = string!(msg);
    let details_json = string!(details_json);
    let maa_msg = MaaMsg::from(&msg, &details_json).unwrap();
    let callback_handler = &mut *(user_data as *mut T);
    callback_handler.handle(maa_msg);
}

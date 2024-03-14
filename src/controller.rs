use std::{fmt::Display, ops::Deref};

use serde::{Deserialize, Serialize};

use crate::{
    error::Error,
    internal::{self, to_cstring},
    maa_bool, CallbackHandler, MaaResult, MaaStatus,
};

#[cfg(feature = "adb")]
pub mod adb;
#[cfg(feature = "dbg")]
pub mod dbg;
#[cfg(feature = "win32")]
pub mod win32;

#[cfg(feature = "adb")]
use self::adb::MaaAdbControllerType;

#[cfg(feature = "custom_controller")]
use crate::custom::custom_controller::MaaCustomController;

#[cfg(feature = "dbg")]
use self::dbg::MaaDbgControllerType;

#[cfg(feature = "win32")]
use self::win32::{MaaWin32ControllerType, MaaWin32Hwnd};

pub use internal::MaaCtrlId;

/// A handle to a controller instance
///
/// # Note
///
/// See [MaaInstance](crate::instance::MaaInstance) for lifetime hints.
pub struct MaaControllerInstance<T> {
    pub(crate) handle: internal::MaaControllerHandle,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Deref for MaaControllerInstance<T> {
    type Target = internal::MaaControllerHandle;

    fn deref(&self) -> &Self::Target {
        &self.handle
    }
}

unsafe impl<T> Send for MaaControllerInstance<T> {}
unsafe impl<T> Sync for MaaControllerInstance<T> {}

impl<T> MaaControllerInstance<T> {
    /// Create a new AdbController
    ///
    /// # Notes
    /// This directly calls MaaAdbControllerCreateV2 since MaaAdbControllerCreate is deprecated
    #[cfg(feature = "adb")]
    pub fn new_adb(
        adb_path: &str,
        address: &str,
        controller_type: MaaAdbControllerType,
        config: &str,
        agent_path: &str,
        handler: Option<T>,
    ) -> Self
    where
        T: CallbackHandler,
    {
        let adb_path = to_cstring(adb_path);
        let address = to_cstring(address);
        let config = to_cstring(config);
        let agent_path = to_cstring(agent_path);

        let handle = unsafe {
            match handler {
                Some(handler) => {
                    let handler = Box::new(handler);
                    let handler = Box::into_raw(handler);

                    internal::MaaAdbControllerCreateV2(
                        adb_path,
                        address,
                        controller_type.into(),
                        config,
                        agent_path,
                        Some(internal::callback_handler::<T>),
                        handler.cast(),
                    )
                }
                None => internal::MaaAdbControllerCreateV2(
                    adb_path,
                    address,
                    controller_type.into(),
                    config,
                    agent_path,
                    None,
                    std::ptr::null_mut(),
                ),
            }
        };

        MaaControllerInstance {
            handle,
            _phantom: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "win32")]
    pub fn new_win32(
        hwnd: MaaWin32Hwnd,
        controller_type: MaaWin32ControllerType,
        handler: Option<T>,
    ) -> Self
    where
        T: CallbackHandler,
    {
        let handle = unsafe {
            match handler {
                Some(handler) => {
                    let handler = Box::new(handler);
                    let handler = Box::into_raw(handler);

                    internal::MaaWin32ControllerCreate(
                        *hwnd,
                        controller_type.into(),
                        Some(internal::callback_handler::<T>),
                        handler.cast(),
                    )
                }
                None => internal::MaaWin32ControllerCreate(
                    *hwnd,
                    controller_type.into(),
                    None,
                    std::ptr::null_mut(),
                ),
            }
        };

        MaaControllerInstance {
            handle,
            _phantom: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "dbg")]
    pub fn new_dbg(
        read_path: &str,
        write_path: &str,
        controller_type: MaaDbgControllerType,
        config: &str,
        handler: Option<T>,
    ) -> Self
    where
        T: CallbackHandler,
    {
        let read_path = to_cstring(read_path);
        let write_path = to_cstring(write_path);
        let config = to_cstring(config);

        let handle = unsafe {
            match handler {
                Some(handler) => {
                    let handler = Box::new(handler);
                    let handler = Box::into_raw(handler);

                    internal::MaaDbgControllerCreate(
                        read_path,
                        write_path,
                        controller_type.into(),
                        config,
                        Some(internal::callback_handler::<T>),
                        handler.cast(),
                    )
                }
                None => internal::MaaDbgControllerCreate(
                    read_path,
                    write_path,
                    controller_type.into(),
                    config,
                    None,
                    std::ptr::null_mut(),
                ),
            }
        };

        MaaControllerInstance {
            handle,
            _phantom: std::marker::PhantomData,
        }
    }

    #[cfg(feature = "custom_controller")]
    pub fn new_custom<C>(controller: C, handler: Option<T>) -> Self
    where
        T: CallbackHandler,
        C: MaaCustomController,
    {
        use crate::custom::custom_controller;

        let controller_api = internal::MaaCustomControllerAPI {
            connect: Some(custom_controller::custom_controller_connect::<C>),
            request_uuid: Some(custom_controller::custom_controller_request_uuid::<C>),
            request_resolution: Some(custom_controller::custom_controller_request_resolution::<C>),
            start_app: Some(custom_controller::custom_controller_start_app::<C>),
            stop_app: Some(custom_controller::custom_controller_stop_app::<C>),
            screencap: Some(custom_controller::custom_controller_screencap::<C>),
            click: Some(custom_controller::custom_controller_click::<C>),
            swipe: Some(custom_controller::custom_controller_swipe::<C>),
            touch_down: Some(custom_controller::custom_controller_touch_down::<C>),
            touch_move: Some(custom_controller::custom_controller_touch_move::<C>),
            touch_up: Some(custom_controller::custom_controller_touch_up::<C>),
            press_key: Some(custom_controller::custom_controller_press_key::<C>),
            input_text: Some(custom_controller::custom_controller_input_text::<C>),
        };

        let controller_api = Box::new(controller_api);
        let controller_api = Box::into_raw(controller_api);

        let handle = unsafe {
            match handler {
                Some(handler) => {
                    let handler = Box::new(handler);
                    let handler = Box::into_raw(handler);

                    internal::MaaCustomControllerCreate(
                        controller_api,
                        &controller as *const C as *mut C as *mut std::ffi::c_void,
                        Some(internal::callback_handler::<T>),
                        handler.cast(),
                    )
                }
                None => internal::MaaCustomControllerCreate(
                    controller_api,
                    &controller as *const C as *mut C as *mut std::ffi::c_void,
                    None,
                    std::ptr::null_mut(),
                ),
            }
        };

        MaaControllerInstance {
            handle,
            _phantom: std::marker::PhantomData,
        }
    }

    pub(crate) fn new_from_handle(handle: internal::MaaControllerHandle) -> Self {
        MaaControllerInstance {
            handle,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn set_controller_option(&self, option: MaaControllerOption) -> MaaResult<()> {
        let key = option.get_inner_key();

        let ret = unsafe {
            match option {
                MaaControllerOption::DefaultAppPackage(ref package) => {
                    let val_size = package.len() as u64;
                    let package = package.as_ptr() as *mut std::os::raw::c_void;
                    let package = package as *mut std::os::raw::c_void;
                    internal::MaaControllerSetOption(self.handle, key, package, val_size)
                }
                MaaControllerOption::DefaultAppPackageEntry(ref package) => {
                    let val_size = package.len() as u64;
                    let package = package.as_ptr() as *mut std::os::raw::c_void;
                    let package = package as *mut std::os::raw::c_void;
                    internal::MaaControllerSetOption(self.handle, key, package, val_size)
                }
                MaaControllerOption::Recording(ref package) => {
                    let val_size = std::mem::size_of::<bool>() as u64;
                    let package = package as *const bool as *mut std::os::raw::c_void;
                    internal::MaaControllerSetOption(self.handle, key, package, val_size)
                }
                MaaControllerOption::ScreenshotTargetLongSide(ref package) => {
                    let val_size = std::mem::size_of::<i32>() as u64;
                    let package = package as *const i32 as *mut std::os::raw::c_void;
                    internal::MaaControllerSetOption(self.handle, key, package, val_size)
                }
                MaaControllerOption::ScreenshotTargetShortSide(ref package) => {
                    let val_size = std::mem::size_of::<i32>() as u64;
                    let package = package as *const i32 as *mut std::os::raw::c_void;
                    internal::MaaControllerSetOption(self.handle, key, package, val_size)
                }
                _ => internal::MaaControllerSetOption(self.handle, key, std::ptr::null_mut(), 0),
            }
        };

        if !maa_bool!(ret) {
            Err(Error::MaaControllerSetOptionError(option))
        } else {
            Ok(())
        }
    }

    pub fn post_connect(&self) -> MaaCtrlId {
        unsafe { internal::MaaControllerPostConnection(self.handle) }
    }

    pub fn post_click(&self, x: i32, y: i32) -> MaaCtrlId {
        unsafe { internal::MaaControllerPostClick(self.handle, x, y) }
    }

    pub fn post_swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration: i32) -> MaaCtrlId {
        unsafe { internal::MaaControllerPostSwipe(self.handle, x1, y1, x2, y2, duration) }
    }

    pub fn post_press_key(&self, keycode: i32) -> MaaCtrlId {
        unsafe { internal::MaaControllerPostPressKey(self.handle, keycode) }
    }

    pub fn post_input_text(&self, text: &str) -> MaaCtrlId {
        let text = to_cstring(text);
        unsafe { internal::MaaControllerPostInputText(self.handle, text) }
    }

    pub fn post_touch_down(&self, contact: i32, x: i32, y: i32, pressure: i32) -> MaaCtrlId {
        unsafe { internal::MaaControllerPostTouchDown(self.handle, contact, x, y, pressure) }
    }

    pub fn post_touch_move(&self, contact: i32, x: i32, y: i32, pressure: i32) -> MaaCtrlId {
        unsafe { internal::MaaControllerPostTouchMove(self.handle, contact, x, y, pressure) }
    }

    pub fn post_touch_up(&self, contact: i32) -> MaaCtrlId {
        unsafe { internal::MaaControllerPostTouchUp(self.handle, contact) }
    }

    pub fn post_screencap(&self) -> MaaCtrlId {
        unsafe { internal::MaaControllerPostScreencap(self.handle) }
    }

    pub fn status(&self, id: MaaCtrlId) -> MaaResult<MaaStatus> {
        let status = unsafe { internal::MaaControllerStatus(self.handle, id) };

        MaaStatus::try_from(status)
    }

    pub fn wait(&self, id: MaaCtrlId) -> MaaResult<MaaStatus> {
        let status = unsafe { internal::MaaControllerWait(self.handle, id) };

        MaaStatus::try_from(status)
    }

    pub fn connected(&self) -> bool {
        unsafe { maa_bool!(internal::MaaControllerConnected(self.handle)) }
    }
}

impl<T> Drop for MaaControllerInstance<T> {
    fn drop(&mut self) {
        unsafe {
            internal::MaaControllerDestroy(self.handle);
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MaaControllerOption {
    Invalid,
    ScreenshotTargetLongSide(i32),
    ScreenshotTargetShortSide(i32),
    DefaultAppPackageEntry(String),
    DefaultAppPackage(String),
    Recording(bool),
}

impl Display for MaaControllerOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaaControllerOption::Invalid => write!(f, "Invalid"),
            MaaControllerOption::ScreenshotTargetLongSide(val) => {
                write!(f, "ScreenshotTargetLongSide: {}", val)
            }
            MaaControllerOption::ScreenshotTargetShortSide(val) => {
                write!(f, "ScreenshotTargetShortSide: {}", val)
            }
            MaaControllerOption::DefaultAppPackageEntry(val) => {
                write!(f, "DefaultAppPackageEntry: {}", val)
            }
            MaaControllerOption::DefaultAppPackage(val) => write!(f, "DefaultAppPackage: {}", val),
            MaaControllerOption::Recording(val) => write!(f, "Recording: {}", val),
        }
    }
}

impl MaaControllerOption {
    fn get_inner_key(&self) -> internal::MaaCtrlOption {
        match self {
            MaaControllerOption::Invalid => internal::MaaCtrlOptionEnum_MaaCtrlOption_Invalid,
            MaaControllerOption::ScreenshotTargetLongSide(_) => {
                internal::MaaCtrlOptionEnum_MaaCtrlOption_ScreenshotTargetLongSide
            }
            MaaControllerOption::ScreenshotTargetShortSide(_) => {
                internal::MaaCtrlOptionEnum_MaaCtrlOption_ScreenshotTargetShortSide
            }
            MaaControllerOption::DefaultAppPackageEntry(_) => {
                internal::MaaCtrlOptionEnum_MaaCtrlOption_DefaultAppPackageEntry
            }
            MaaControllerOption::DefaultAppPackage(_) => {
                internal::MaaCtrlOptionEnum_MaaCtrlOption_DefaultAppPackage
            }
            MaaControllerOption::Recording(_) => {
                internal::MaaCtrlOptionEnum_MaaCtrlOption_Recording
            }
        }
    }
}

//! > **Rust bindings for MaaFramework**
//!
//! This crate provides Rust bindings for [MaaFramework](https://github.com/MaaAssistantArknights/MaaFramework).
//! On top of the raw bindings generated by bindgen, we provide a safe and more rust-friendly wrapper for use.
//!
//! ## Pre-requisites
//!
//! This crate utilizes cmake to find and link to the MaaFramework library. You will need to have cmake installed on your system and make sure that MaaFramework is installed in a place where cmake can find it.
//! In addition, you will also need the MaaFramework library installed on your system to run any tests or binaries that use this crate.
//!
//! ## Usage
//!
//! Refer to the concerning struct for usage examples. Furthermore, you should check the MaaFramework repo to get a more in-depth understanding of the API.
//!
//! If you have no idea where to start, you can check the [instance] module for starter.
//!
//! ## Features
//!
//! - `internal`: Enable internal API for MaaFramework. This enables you to directly access the raw bindings.
//! - `toolkit`: Enable MaaToolkit.
//! - `sync_context`: Enable sync context for MaaFramework.
//! - `adb`: Enable adb controller for MaaFramework.
//! - `win32`: Enable win32 controller for MaaFramework.
//! - `dbg`: Enable debug controller for MaaFramework. This is most likely not needed.
//! - `custom_recognizer`: Enable custom recognizer for MaaFramework.
//! - `custom_controller`: Enable custom controller for MaaFramework.
//! - `custom_action`: Enable custom action for MaaFramework.
//! - `custom`: Enable all custom features for MaaFramework.
//!
//! The default features include all features so you might want to disable some of them if you don't need them.

#![feature(doc_cfg)]

use std::{ffi::c_void, fmt::Display, ptr::null_mut};

use msg::MaaMsg;
use serde::{Deserialize, Serialize};

#[cfg(feature = "internal")]
#[doc(cfg(feature = "internal"))]
pub mod internal;

#[cfg(not(feature = "internal"))]
mod internal;

#[cfg(feature = "toolkit")]
#[doc(cfg(feature = "toolkit"))]
pub mod toolkit;

pub mod custom;

#[cfg(feature = "sync_context")]
#[doc(cfg(feature = "sync_context"))]
pub mod sync_context;

pub mod buffer;
pub mod controller;
pub mod error;
pub mod instance;
pub mod msg;
pub mod resource;

use error::Error;

pub type MaaResult<T> = Result<T, error::Error>;

pub enum MaaStatus {
    Invalid,
    Pending,
    Running,
    Success,
    Failed,
}

impl TryFrom<internal::MaaStatus> for MaaStatus {
    type Error = error::Error;

    fn try_from(status: internal::MaaStatus) -> Result<Self, Self::Error> {
        match status {
            internal::MaaStatusEnum_MaaStatus_Invalid => Ok(MaaStatus::Invalid),
            internal::MaaStatusEnum_MaaStatus_Pending => Ok(MaaStatus::Pending),
            internal::MaaStatusEnum_MaaStatus_Running => Ok(MaaStatus::Running),
            internal::MaaStatusEnum_MaaStatus_Success => Ok(MaaStatus::Success),
            internal::MaaStatusEnum_MaaStatus_Failed => Ok(MaaStatus::Failed),
            _ => Err(error::Error::MaaStatusConversionError(status)),
        }
    }
}

/// The callback handler trait.
///
/// This trait is used to handle the callback from MaaFramework.
pub trait CallbackHandler {
    fn handle(&mut self, msg: MaaMsg);
}

pub fn maa_version() -> String {
    let version = unsafe { internal::MaaVersion() };

    string!(version)
}

#[repr(C)]
#[derive(Debug, Serialize, Deserialize)]
pub enum MaaLoggingLevel {
    Off = 0,
    Fatal = 1,
    Error = 2,
    Warn = 3,
    Info = 4,
    Debug = 5,
    Trace = 6,
    All = 7,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MaaGlobalOption {
    Invalid,
    LogDir(String),
    SaveDraw(bool),
    Recording(bool),
    StdoutLevel(MaaLoggingLevel),
    ShowHitDraw(bool),
    DebugMessage(bool),
}

impl Display for MaaGlobalOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaaGlobalOption::Invalid => write!(f, "Invalid"),
            MaaGlobalOption::LogDir(dir) => write!(f, "LogDir: {}", dir),
            MaaGlobalOption::SaveDraw(save) => write!(f, "SaveDraw: {}", save),
            MaaGlobalOption::Recording(recording) => write!(f, "Recording: {}", recording),
            MaaGlobalOption::StdoutLevel(level) => write!(f, "StdoutLevel: {:?}", level),
            MaaGlobalOption::ShowHitDraw(show) => write!(f, "ShowHitDraw: {}", show),
            MaaGlobalOption::DebugMessage(debug) => write!(f, "DebugMessage: {}", debug),
        }
    }
}

impl MaaGlobalOption {
    fn get_inner_key(&self) -> internal::MaaGlobalOption {
        match self {
            MaaGlobalOption::Invalid => internal::MaaGlobalOptionEnum_MaaGlobalOption_Invalid,
            MaaGlobalOption::LogDir(_) => internal::MaaGlobalOptionEnum_MaaGlobalOption_LogDir,
            MaaGlobalOption::SaveDraw(_) => internal::MaaGlobalOptionEnum_MaaGlobalOption_SaveDraw,
            MaaGlobalOption::Recording(_) => {
                internal::MaaGlobalOptionEnum_MaaGlobalOption_Recording
            }
            MaaGlobalOption::StdoutLevel(_) => {
                internal::MaaGlobalOptionEnum_MaaGlobalOption_StdoutLevel
            }
            MaaGlobalOption::ShowHitDraw(_) => {
                internal::MaaGlobalOptionEnum_MaaGlobalOption_ShowHitDraw
            }
            MaaGlobalOption::DebugMessage(_) => {
                internal::MaaGlobalOptionEnum_MaaGlobalOption_DebugMessage
            }
        }
    }
}

pub fn set_global_option(option: MaaGlobalOption) -> MaaResult<()> {
    let key = option.get_inner_key();

    let ret = match option {
        MaaGlobalOption::LogDir(ref dir) => {
            let c_dir = dir.as_ptr() as *mut c_void;
            let len = dir.len() as u64;
            unsafe { internal::MaaSetGlobalOption(key, c_dir, len) }
        }
        MaaGlobalOption::SaveDraw(ref save) => {
            let val_size = std::mem::size_of::<bool>() as u64;
            let value = save as *const bool as *mut c_void;
            unsafe { internal::MaaSetGlobalOption(key, value, val_size) }
        }
        MaaGlobalOption::Recording(ref recording) => {
            let val_size = std::mem::size_of::<bool>() as u64;
            let value = recording as *const bool as *mut c_void;
            unsafe { internal::MaaSetGlobalOption(key, value, val_size) }
        }
        MaaGlobalOption::StdoutLevel(ref level) => {
            let val_size = std::mem::size_of::<MaaLoggingLevel>() as u64;
            let value = level as *const MaaLoggingLevel as *mut c_void;
            unsafe { internal::MaaSetGlobalOption(key, value, val_size) }
        }
        MaaGlobalOption::ShowHitDraw(ref show) => {
            let val_size = std::mem::size_of::<bool>() as u64;
            let value = show as *const bool as *mut c_void;
            unsafe { internal::MaaSetGlobalOption(key, value, val_size) }
        }
        MaaGlobalOption::DebugMessage(ref debug) => {
            let val_size = std::mem::size_of::<bool>() as u64;
            let value = debug as *const bool as *mut c_void;
            unsafe { internal::MaaSetGlobalOption(key, value, val_size) }
        }
        _ => unsafe { internal::MaaSetGlobalOption(key, null_mut(), 0) },
    };

    maa_bool!(ret, MaaSetGlobalOptionError, option)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_set_global_option() {
        super::set_global_option(super::MaaGlobalOption::LogDir("test".to_string())).unwrap();

        super::set_global_option(super::MaaGlobalOption::SaveDraw(true)).unwrap();

        super::set_global_option(super::MaaGlobalOption::StdoutLevel(
            crate::MaaLoggingLevel::Debug,
        ))
        .unwrap();
    }
}

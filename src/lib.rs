//! # Rust bindings for MaaFramework

use std::{ffi::c_void, fmt::Display, ptr::null_mut};

use serde::{Deserialize, Serialize};

#[cfg(feature = "internal")]
pub mod internal;

#[cfg(not(feature = "internal"))]
mod internal;

#[cfg(feature = "toolkit")]
pub mod toolkit;

pub mod custom;

#[cfg(feature = "sync_context")]
pub mod sync_context;

pub mod buffer;
pub mod controller;
pub mod error;
pub mod instance;
pub mod resource;
pub mod msg;

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

pub trait CallbackHandler {
    fn handle(&self, msg: &str, details_json: &str);
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
    ShowHitDraw(bool)
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
        }
    }
}

impl MaaGlobalOption {
    fn get_inner_key(&self) -> internal::MaaGlobalOption {
        match self {
            MaaGlobalOption::Invalid => internal::MaaGlobalOptionEnum_MaaGlobalOption_Invalid,
            MaaGlobalOption::LogDir(_) => internal::MaaGlobalOptionEnum_MaaGlobalOption_LogDir,
            MaaGlobalOption::SaveDraw(_) => internal::MaaGlobalOptionEnum_MaaGlobalOption_SaveDraw,
            MaaGlobalOption::Recording(_) => internal::MaaGlobalOptionEnum_MaaGlobalOption_Recording,
            MaaGlobalOption::StdoutLevel(_) => internal::MaaGlobalOptionEnum_MaaGlobalOption_StdoutLevel,
            MaaGlobalOption::ShowHitDraw(_) => internal::MaaGlobalOptionEnum_MaaGlobalOption_ShowHitDraw,
        }

    }
}

pub fn set_global_option(option: MaaGlobalOption) -> MaaResult<()> {

    let key = option.get_inner_key();

    let ret = match option {
        MaaGlobalOption::LogDir(ref dir) => {
            let c_dir = dir.as_ptr() as *mut c_void;
            let len = dir.len() as u64;
            unsafe { internal::MaaSetGlobalOption(key, c_dir,len) }
        },
        MaaGlobalOption::SaveDraw(ref save) => {
            let val_size = std::mem::size_of::<bool>() as u64;
            let value = save as *const bool as *mut c_void;
            unsafe { internal::MaaSetGlobalOption(key, value,val_size) }
        },
        MaaGlobalOption::Recording(ref recording) => {
            let val_size = std::mem::size_of::<bool>() as u64;
            let value = recording as *const bool as *mut c_void;
            unsafe { internal::MaaSetGlobalOption(key, value,val_size) }
        },
        MaaGlobalOption::StdoutLevel(ref level) => {
            let val_size = std::mem::size_of::<MaaLoggingLevel>() as u64;
            let value = level as *const MaaLoggingLevel as *mut c_void;
            unsafe { internal::MaaSetGlobalOption(key, value, val_size) }
        },
        MaaGlobalOption::ShowHitDraw(ref show) => {
            let val_size = std::mem::size_of::<bool>() as u64;
            let value = show as *const bool as *mut c_void;
            unsafe { internal::MaaSetGlobalOption(key, value, val_size) }
        },
        _ => {
            unsafe { internal::MaaSetGlobalOption(key, null_mut(), 0) }
        }
    };

    maa_bool!(ret, MaaSetGlobalOptionError, option)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_set_global_option() {
        super::set_global_option(
            super::MaaGlobalOption::LogDir("test".to_string())
        ).unwrap();

        super::set_global_option(
            super::MaaGlobalOption::SaveDraw(true)
        ).unwrap();

        super::set_global_option(
            super::MaaGlobalOption::StdoutLevel(crate::MaaLoggingLevel::Debug)
        ).unwrap();
    }
}
use std::{ffi::c_void, fmt::Display, ptr::null_mut};

use serde::{Deserialize, Serialize};

use crate::{
    buffer::{
        image_list_buffer::MaaImageListBuffer, rect_buffer::MaaRectBuffer,
        string_buffer::MaaStringBuffer,
    },
    internal::{self, MaaQueryRecognitionDetail},
    maa_bool, Error, MaaResult,
};

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RunningDetail {
    pub reco_id: i64,
    pub successful: bool,
}

pub fn query_running_detail(run_id: i64) -> RunningDetail {
    let mut reco_id: i64 = 0;
    let mut successful: u8 = 0;

    unsafe {
        internal::MaaQueryRunningDetail(run_id, &mut reco_id, &mut successful);
    }

    let successful = maa_bool!(successful);
    RunningDetail {
        reco_id,
        successful,
    }
}

pub struct RecognitionDetail {
    pub hit: bool,
    pub hit_box: MaaRectBuffer,
    pub detail_json: MaaStringBuffer,
    pub draws: MaaImageListBuffer,
}

pub fn query_recognition_detail(reco_id: i64) -> RecognitionDetail {
    let mut hit: u8 = 0;
    let hit_box: MaaRectBuffer = Default::default();
    let detail_json: MaaStringBuffer = Default::default();
    let draws: MaaImageListBuffer = Default::default();

    unsafe {
        MaaQueryRecognitionDetail(
            reco_id,
            &mut hit,
            hit_box.handle,
            detail_json.handle,
            draws.handle,
        );
    }

    let hit = maa_bool!(hit);

    RecognitionDetail {
        hit,
        hit_box,
        detail_json,
        draws,
    }
}

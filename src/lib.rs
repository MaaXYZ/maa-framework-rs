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
pub mod diff_task;
pub mod error;
pub mod instance;
pub mod msg;
pub mod resource;
pub mod utility;

use error::Error;

pub type MaaResult<T> = Result<T, error::Error>;

#[derive(Debug, Serialize, Deserialize)]
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

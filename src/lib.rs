//! # Rust bindings for MaaFramework

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

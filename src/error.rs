use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    controller::MaaControllerOption,
    instance::{MaaInstOption, MaaTaskId},
    resource::MaaResOption,
    MaaGlobalOption,
};

#[derive(Error, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Error {
    #[error("Maa fails to set global option {0}")]
    MaaSetGlobalOptionError(MaaGlobalOption),

    #[error("MaaToolkit failed to init")]
    MaaToolkitInitError,

    #[error("MaaStatus conversion error: {0}")]
    MaaStatusConversionError(i32),

    #[error("MaaAdbControllerType conversion error: {0}")]
    MaaAdbControllerTypeConversionError(i32),

    #[error("MaaWin32ControllerType conversion error: {0}")]
    MaaWin32ControllerTypeConversionError(i32),

    #[error("MaaDbgControllerType conversion error: {0}")]
    MaaDbgControllerTypeConversionError(i32),

    #[error("MaaController fails to set option {0}")]
    MaaControllerSetOptionError(MaaControllerOption),

    #[error("MaaResource fails to set option {0}")]
    MaaResourceSetOptionError(MaaResOption),

    #[error("MaaInstance fails to set option {0}")]
    MaaInstanceSetOptionError(MaaInstOption),

    #[error("MaaInstance fails to bind resource")]
    MaaInstanceBindResourceError,

    #[error("MaaInstance fails to bind controller")]
    MaaInstanceBindControllerError,

    #[error("MaaInstance fails to set task param {0}")]
    MaaInstanceSetTaskParamError(MaaTaskId),

    #[error("MaaInstance fails to stop")]
    MaaInstanceStopError,

    #[error("MaaInstance fails to register custom recognizer {0}")]
    MaaInstanceRegisterCustomRecognizerError(String),

    #[error("MaaInstance fails to unregister custom recognizer {0}")]
    MaaInstanceUnregisterCustomRecognizerError(String),

    #[error("MaaInstance fails to clear custom recognizer")]
    MaaInstanceClearCustomRecognizerError,

    #[error("MaaInstance fails to register custom action {0}")]
    MaaInstanceRegisterCustomActionError(String),

    #[error("MaaInstance fails to unregister custom action {0}")]
    MaaInstanceUnregisterCustomActionError(String),

    #[error("MaaInstance fails to clear custom action")]
    MaaInstanceClearCustomActionError,

    #[error("MaaSyncContext fails to run task: {0}")]
    MaaSyncContextRunTaskError(String),

    #[error("MaaSyncContext fails to run recognizer: {0}")]
    MaaSyncContextRunRecognizerError(String),

    #[error("MaaSyncContext fails to run action: {0}")]
    MaaSyncContextRunActionError(String),

    #[error("MaaSyncContext fails to click")]
    MaaSyncContextClickError,

    #[error("MaaSyncContext fails to swipe")]
    MaaSyncContextSwipeError,

    #[error("MaaSyncContext fails to press key {0}")]
    MaaSyncContextPressKeyError(i32),

    #[error("MaaSyncContext fails to input text {0}")]
    MaaSyncContextInputTextError(String),

    #[error("MaaSyncContext fails to touch down")]
    MaaSyncContextTouchDownError,

    #[error("MaaSyncContext fails to touch move")]
    MaaSyncContextTouchMoveError,

    #[error("MaaSyncContext fails to touch up")]
    MaaSyncContextTouchUpError,

    #[error("MaaSyncContext fails to screencap")]
    MaaSyncContextScreencapError,

    #[error("MaaSyncContext fails to get task result")]
    MaaSyncContextGetTaskResultError,

    #[error("MaaResource fails to get hash")]
    MaaResourceGetHashError,

    #[error("MaaResource fails to get task list")]
    MaaResourceGetTaskListError,

    #[error("MaaResource fails to clear")]
    MaaResourceClearError,

    #[error("Maa fails to set string buffeer {0}")]
    MaaSetStringError(String),

    #[error("MaaToolkit fails to register custom recognizer executor")]
    MaaToolkitRegisterCustomRecognizerExecutorError,

    #[error("MaaToolkit fails to unregister custom recognizer executor")]
    MaaToolkitUnregisterCustomRecognizerExecutorError,

    #[error("MaaToolkit fails to find device")]
    MaaToolkitPostFindDeviceError,

    #[error("(De)serialize error: {0}")]
    SerdeError(String),
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeError(e.to_string())
    }
}

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::MaaResult;

#[derive(Debug, Serialize, Deserialize)]
pub struct MaaMsgResource {
    pub id: i32,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaaMsgUUID {
    pub uuid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaaMsgResolution {
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaaMsgConnect {
    pub uuid: String,
    pub resolution: MaaMsgResolution,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaaMsgConnectFailed {
    pub why: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaaMsgAction {
    pub id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaaMsgTask {
    pub id: i32,
    pub entry: String,
    pub name: String,
    pub uuid: String,
    pub hash: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaaMsgTaskFocus {
    pub id: i32,
    pub entry: String,
    pub name: String,
    pub uuid: String,
    pub hash: String,
    pub recognition: Value,
    pub run_times: i32,
    pub last_time: String,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MaaMsgTaskDebug {
    pub id: i32,
    pub entry: String,
    pub uuid: String,
    pub hash: String,
    pub name: String,
    pub latest_hit: String,
    pub recognition: Value,
    pub run_times: i32,
    pub status: String,
}

#[non_exhaustive]
pub enum MaaMsg {
    Invalid,

    ResourceStartLoading(MaaMsgResource),
    ResourceLoadingCompleted(MaaMsgResource),
    ResourceLoadingFailed(MaaMsgResource),

    ControllerUUIDGot(MaaMsgUUID),
    ControllerUUIDGetFailed,

    ControllerResolutionGot(MaaMsgResolution),
    ControllerResolutionGetFailed,

    ControllerScreencapInited,
    ControllerScreencapInitFailed,
    ControllerTouchInputInited,
    ControllerTouchInputInitFailed,
    ControllerKeyInputInited,
    ControllerKeyInputInitFailed,

    ControllerConnectSuccess(MaaMsgConnect),
    ControllerConnectFailed(MaaMsgConnectFailed),

    ControllerActionStarted(MaaMsgAction),
    ControllerActionCompleted(MaaMsgAction),
    ControllerActionFailed(MaaMsgAction),

    TaskStarted(MaaMsgTask),
    TaskCompleted(MaaMsgTask),
    TaskFailed(MaaMsgTask),
    TaskStopped(MaaMsgTask),

    TaskFocusHit(MaaMsgTaskFocus),
    TaskFocusRunout(MaaMsgTaskFocus),
    TaskFocusCompleted(MaaMsgTaskFocus),

    TaskDebugReadyToTun(MaaMsgTaskDebug),
    TaskDebugRunout(MaaMsgTaskDebug),
    TaskDebugCompleted(MaaMsgTaskDebug),
    TaskDebugListToRecognize,
    TaskDebugHit
}

impl MaaMsg {
    pub fn from(msg: &str, details: &str) -> MaaResult<Self> {
        let value = match msg {
            "Resource.StartLoading" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::ResourceStartLoading(details)
            }
            "Resource.LoadingCompleted" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::ResourceLoadingCompleted(details)
            }
            "Resource.LoadingFailed" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::ResourceLoadingFailed(details)
            }
            "Controller.UUIDGot" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::ControllerUUIDGot(details)
            }
            "Controller.UUIDGetFailed" => MaaMsg::ControllerUUIDGetFailed,
            "Controller.ResolutionGot" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::ControllerResolutionGot(details)
            }
            "Controller.ResolutionGetFailed" => MaaMsg::ControllerResolutionGetFailed,
            "Controller.ScreencapInited" => MaaMsg::ControllerScreencapInited,
            "Controller.ScreencapInitFailed" => MaaMsg::ControllerScreencapInitFailed,
            "Controller.TouchInputInited" => MaaMsg::ControllerTouchInputInited,
            "Controller.TouchInputInitFailed" => MaaMsg::ControllerTouchInputInitFailed,
            "Controller.KeyInputInited" => MaaMsg::ControllerKeyInputInited,
            "Controller.KeyInputInitFailed" => MaaMsg::ControllerKeyInputInitFailed,
            "Controller.ConnectSuccess" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::ControllerConnectSuccess(details)
            }
            "Controller.ConnectFailed" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::ControllerConnectFailed(details)
            }
            "Controller.ActionStarted" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::ControllerActionStarted(details)
            }
            "Controller.ActionCompleted" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::ControllerActionCompleted(details)
            }
            "Controller.ActionFailed" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::ControllerActionFailed(details)
            }
            "Task.Started" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::TaskStarted(details)
            }
            "Task.Completed" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::TaskCompleted(details)
            }
            "Task.Failed" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::TaskFailed(details)
            }
            "Task.Stopped" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::TaskStopped(details)
            }
            "Task.Focus.ReadyToRun" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::TaskFocusHit(details)
            }
            "Task.Focus.Runout" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::TaskFocusRunout(details)
            }
            "Task.Focus.Completed" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::TaskFocusCompleted(details)
            },
            "Task.Debug.ReadyToRun" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::TaskDebugReadyToTun(details)
            }
            "Task.Debug.Runout" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::TaskDebugRunout(details)
            }
            "Task.Debug.Completed" => {
                let details = serde_json::from_str(details)?;
                MaaMsg::TaskDebugCompleted(details)
            }
            "Task.Debug.ListToRecognize" => MaaMsg::TaskDebugListToRecognize,
            "Task.Debug.Hit" => MaaMsg::TaskDebugHit,
            _ => MaaMsg::Invalid,
        };

        Ok(value)
    }
}

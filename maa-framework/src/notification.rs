//! Structured notification parsing for event callbacks.
//!
//! This module provides typed structures and parsing utilities for the event
//! notifications from MaaFramework. Instead of manually parsing JSON strings,
//! use these helpers to work with strongly-typed event data.
//!
//! # Example
//!
//! ```
//! use maa_framework::notification::{self, NotificationType};
//! use maa_framework::tasker::Tasker;
//!
//! fn example(tasker: &Tasker) -> maa_framework::error::MaaResult<()> {
//!     tasker.add_sink(|message, details| {
//!         let noti_type = notification::parse_type(message);
//!         
//!         if message.starts_with("Resource.Loading") {
//!             if let Some(detail) = notification::parse_resource_loading(details) {
//!                 println!("Resource {} loading: {:?}", detail.res_id, noti_type);
//!             }
//!         }
//!     })?;
//!     Ok(())
//! }
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value;

// === Notification Type ===

/// Type of notification event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationType {
    /// Operation is starting
    Starting,
    /// Operation succeeded
    Succeeded,
    /// Operation failed
    Failed,
    /// Unknown notification type
    Unknown,
}

impl NotificationType {
    /// Check if this is a starting notification.
    pub fn is_starting(&self) -> bool {
        matches!(self, Self::Starting)
    }

    /// Check if this is a succeeded notification.
    pub fn is_succeeded(&self) -> bool {
        matches!(self, Self::Succeeded)
    }

    /// Check if this is a failed notification.
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed)
    }

    /// Check if the operation is complete (succeeded or failed).
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed)
    }
}

impl From<&str> for NotificationType {
    fn from(s: &str) -> Self {
        if s.ends_with(".Starting") {
            NotificationType::Starting
        } else if s.ends_with(".Succeeded") {
            NotificationType::Succeeded
        } else if s.ends_with(".Failed") {
            NotificationType::Failed
        } else {
            NotificationType::Unknown
        }
    }
}

// === Event Detail Structures ===

/// Resource loading event detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLoadingDetail {
    /// Resource ID
    pub res_id: i64,
    /// Resource hash
    pub hash: String,
    /// Path being loaded
    pub path: String,
}

/// Controller action event detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerActionDetail {
    /// Controller ID
    pub ctrl_id: i64,
    /// Device UUID
    pub uuid: String,
    /// Action name
    pub action: String,
    /// Action parameters
    #[serde(default)]
    pub param: Value,
}

/// Tasker task event detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskerTaskDetail {
    /// Task ID
    pub task_id: i64,
    /// Entry node name
    pub entry: String,
    /// Device UUID
    #[serde(default)]
    pub uuid: String,
    /// Resource hash
    #[serde(default)]
    pub hash: String,
}

/// Next list item for node traversal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextListItem {
    /// Node name
    pub name: String,
    /// Whether to jump back after execution
    #[serde(default)]
    pub jump_back: bool,
    /// Whether this is an anchor node
    #[serde(default)]
    pub anchor: bool,
}

/// Node next list event detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeNextListDetail {
    /// Task ID
    pub task_id: i64,
    /// Current node name
    pub name: String,
    /// List of next nodes
    #[serde(default)]
    pub list: Vec<NextListItem>,
    /// Focus configuration
    #[serde(default)]
    pub focus: Value,
}

/// Node recognition event detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeRecognitionDetail {
    /// Task ID
    pub task_id: i64,
    /// Recognition ID
    pub reco_id: i64,
    /// Node name
    pub name: String,
    /// Focus configuration
    #[serde(default)]
    pub focus: Value,
}

/// Node action event detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeActionDetail {
    /// Task ID
    pub task_id: i64,
    /// Action ID
    pub action_id: i64,
    /// Node name
    pub name: String,
    /// Focus configuration
    #[serde(default)]
    pub focus: Value,
}

/// Node pipeline node event detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePipelineNodeDetail {
    /// Task ID
    pub task_id: i64,
    /// Node ID
    pub node_id: i64,
    /// Node name
    pub name: String,
    /// Focus configuration
    #[serde(default)]
    pub focus: Value,
}

// === Message Constants ===

/// Notification message constants.
pub mod msg {
    // Resource events
    pub const RESOURCE_LOADING_STARTING: &str = "Resource.Loading.Starting";
    pub const RESOURCE_LOADING_SUCCEEDED: &str = "Resource.Loading.Succeeded";
    pub const RESOURCE_LOADING_FAILED: &str = "Resource.Loading.Failed";

    // Controller events
    pub const CONTROLLER_ACTION_STARTING: &str = "Controller.Action.Starting";
    pub const CONTROLLER_ACTION_SUCCEEDED: &str = "Controller.Action.Succeeded";
    pub const CONTROLLER_ACTION_FAILED: &str = "Controller.Action.Failed";

    // Tasker events
    pub const TASKER_TASK_STARTING: &str = "Tasker.Task.Starting";
    pub const TASKER_TASK_SUCCEEDED: &str = "Tasker.Task.Succeeded";
    pub const TASKER_TASK_FAILED: &str = "Tasker.Task.Failed";

    // Node pipeline events
    pub const NODE_PIPELINE_NODE_STARTING: &str = "Node.PipelineNode.Starting";
    pub const NODE_PIPELINE_NODE_SUCCEEDED: &str = "Node.PipelineNode.Succeeded";
    pub const NODE_PIPELINE_NODE_FAILED: &str = "Node.PipelineNode.Failed";

    // Node recognition events
    pub const NODE_RECOGNITION_STARTING: &str = "Node.Recognition.Starting";
    pub const NODE_RECOGNITION_SUCCEEDED: &str = "Node.Recognition.Succeeded";
    pub const NODE_RECOGNITION_FAILED: &str = "Node.Recognition.Failed";

    // Node action events
    pub const NODE_ACTION_STARTING: &str = "Node.Action.Starting";
    pub const NODE_ACTION_SUCCEEDED: &str = "Node.Action.Succeeded";
    pub const NODE_ACTION_FAILED: &str = "Node.Action.Failed";

    // Node next list events
    pub const NODE_NEXT_LIST_STARTING: &str = "Node.NextList.Starting";
    pub const NODE_NEXT_LIST_SUCCEEDED: &str = "Node.NextList.Succeeded";
    pub const NODE_NEXT_LIST_FAILED: &str = "Node.NextList.Failed";

    // Node recognition node trace events
    pub const NODE_RECOGNITION_NODE_STARTING: &str = "Node.RecognitionNode.Starting";
    pub const NODE_RECOGNITION_NODE_SUCCEEDED: &str = "Node.RecognitionNode.Succeeded";
    pub const NODE_RECOGNITION_NODE_FAILED: &str = "Node.RecognitionNode.Failed";

    // Node action node trace events
    pub const NODE_ACTION_NODE_STARTING: &str = "Node.ActionNode.Starting";
    pub const NODE_ACTION_NODE_SUCCEEDED: &str = "Node.ActionNode.Succeeded";
    pub const NODE_ACTION_NODE_FAILED: &str = "Node.ActionNode.Failed";
}

// === Parse Functions ===

/// Parse notification type from message string.
///
/// # Example
/// ```
/// use maa_framework::notification::{self, NotificationType};
///
/// let noti_type = notification::parse_type("Resource.Loading.Succeeded");
/// assert_eq!(noti_type, NotificationType::Succeeded);
/// ```
pub fn parse_type(msg: &str) -> NotificationType {
    NotificationType::from(msg)
}

/// Parse resource loading event detail from JSON.
pub fn parse_resource_loading(details: &str) -> Option<ResourceLoadingDetail> {
    serde_json::from_str(details).ok()
}

/// Parse controller action event detail from JSON.
pub fn parse_controller_action(details: &str) -> Option<ControllerActionDetail> {
    serde_json::from_str(details).ok()
}

/// Parse tasker task event detail from JSON.
pub fn parse_tasker_task(details: &str) -> Option<TaskerTaskDetail> {
    serde_json::from_str(details).ok()
}

/// Parse node next list event detail from JSON.
pub fn parse_node_next_list(details: &str) -> Option<NodeNextListDetail> {
    serde_json::from_str(details).ok()
}

/// Parse node recognition event detail from JSON.
pub fn parse_node_recognition(details: &str) -> Option<NodeRecognitionDetail> {
    serde_json::from_str(details).ok()
}

/// Parse node action event detail from JSON.
pub fn parse_node_action(details: &str) -> Option<NodeActionDetail> {
    serde_json::from_str(details).ok()
}

/// Parse node pipeline node event detail from JSON.
pub fn parse_node_pipeline_node(details: &str) -> Option<NodePipelineNodeDetail> {
    serde_json::from_str(details).ok()
}

// === Context Event ===

/// Enum representing parsed Context events.
#[derive(Debug, Clone)]
pub enum ContextEvent {
    NodeNextList(NotificationType, NodeNextListDetail),
    NodeRecognition(NotificationType, NodeRecognitionDetail),
    NodeAction(NotificationType, NodeActionDetail),
    NodePipelineNode(NotificationType, NodePipelineNodeDetail),
    NodeRecognitionNode(NotificationType, NodePipelineNodeDetail),
    NodeActionNode(NotificationType, NodePipelineNodeDetail),
    Unknown(String, Value),
}

impl ContextEvent {
    /// Parse a raw notification into a strongly-typed ContextEvent.
    pub fn from_notification(msg: &str, details: &str) -> Option<Self> {
        let noti_type = NotificationType::from(msg);

        let parse_json = || -> Option<Value> { serde_json::from_str(details).ok() };

        if msg.starts_with("Node.NextList") {
            let detail = parse_node_next_list(details)?;
            return Some(ContextEvent::NodeNextList(noti_type, detail));
        }

        if msg.starts_with("Node.Recognition.") {
            let detail = parse_node_recognition(details)?;
            return Some(ContextEvent::NodeRecognition(noti_type, detail));
        }

        if msg.starts_with("Node.Action.") {
            let detail = parse_node_action(details)?;
            return Some(ContextEvent::NodeAction(noti_type, detail));
        }

        if msg.starts_with("Node.PipelineNode") {
            let detail = parse_node_pipeline_node(details)?;
            return Some(ContextEvent::NodePipelineNode(noti_type, detail));
        }

        if msg.starts_with("Node.RecognitionNode") {
            let detail = parse_node_pipeline_node(details)?;
            return Some(ContextEvent::NodeRecognitionNode(noti_type, detail));
        }

        if msg.starts_with("Node.ActionNode") {
            let detail = parse_node_pipeline_node(details)?;
            return Some(ContextEvent::NodeActionNode(noti_type, detail));
        }

        Some(ContextEvent::Unknown(
            msg.to_string(),
            parse_json().unwrap_or(Value::Null),
        ))
    }
}

// === Event Sink Traits ===

/// Trait for handling resource events.
pub trait ResourceEventHandler: Send + Sync {
    /// Called when a resource loading event occurs.
    fn on_resource_loading(&self, _noti_type: NotificationType, _detail: ResourceLoadingDetail) {}

    /// Called when an unknown notification is received.
    fn on_unknown(&self, _msg: &str, _details: &Value) {}
}

/// Trait for handling controller events.
pub trait ControllerEventHandler: Send + Sync {
    /// Called when a controller action event occurs.
    fn on_controller_action(&self, _noti_type: NotificationType, _detail: ControllerActionDetail) {}

    /// Called when an unknown notification is received.
    fn on_unknown(&self, _msg: &str, _details: &Value) {}
}

/// Trait for handling tasker events.
pub trait TaskerEventHandler: Send + Sync {
    /// Called when a tasker task event occurs.
    fn on_tasker_task(&self, _noti_type: NotificationType, _detail: TaskerTaskDetail) {}

    /// Called when an unknown notification is received.
    fn on_unknown(&self, _msg: &str, _details: &Value) {}
}

/// Trait for handling context/node events.
pub trait ContextEventHandler: Send + Sync {
    /// Called when a node next list event occurs.
    fn on_node_next_list(&self, _noti_type: NotificationType, _detail: NodeNextListDetail) {}

    /// Called when a node recognition event occurs.
    fn on_node_recognition(&self, _noti_type: NotificationType, _detail: NodeRecognitionDetail) {}

    /// Called when a node action event occurs.
    fn on_node_action(&self, _noti_type: NotificationType, _detail: NodeActionDetail) {}

    /// Called when a node pipeline node event occurs.
    fn on_node_pipeline_node(&self, _noti_type: NotificationType, _detail: NodePipelineNodeDetail) {
    }

    /// Called when a node recognition node (trace) event occurs.
    fn on_node_recognition_node(
        &self,
        _noti_type: NotificationType,
        _detail: NodePipelineNodeDetail,
    ) {
    }

    /// Called when a node action node (trace) event occurs.
    fn on_node_action_node(&self, _noti_type: NotificationType, _detail: NodePipelineNodeDetail) {}

    /// Called when an unknown notification is received.
    fn on_unknown(&self, _msg: &str, _details: &Value) {}
}

// === MaaEvent ===

/// Unified event enum for all framework notifications.
///
/// This enum encapsulates all possible event types emitted by the framework, providing
/// a type-safe way to handle notifications from `Tasker`, `Controller`, or `Resource`.
/// Each variant wraps a detailed structure containing relevant information about the event.
///
/// # See Also
/// * [`EventSink`](crate::event_sink::EventSink) - The trait for receiving these events.
/// * [`Tasker::add_event_sink`](crate::tasker::Tasker::add_event_sink) - Registering an event sink.
#[derive(Debug, Clone)]
pub enum MaaEvent {
    // --- Resource Events ---
    /// Triggered when a resource starts loading.
    ResourceLoadingStarting(ResourceLoadingDetail),
    /// Triggered when a resource is successfully loaded.
    ResourceLoadingSucceeded(ResourceLoadingDetail),
    /// Triggered when resource loading fails.
    ResourceLoadingFailed(ResourceLoadingDetail),

    // --- Controller Events ---
    /// Triggered before a controller performs an action (e.g., click, swipe).
    ControllerActionStarting(ControllerActionDetail),
    /// Triggered after a controller action completes successfully.
    ControllerActionSucceeded(ControllerActionDetail),
    /// Triggered if a controller action fails.
    ControllerActionFailed(ControllerActionDetail),

    // --- Tasker Events ---
    /// Triggered when a task begins execution.
    TaskerTaskStarting(TaskerTaskDetail),
    /// Triggered when a task completes successfully.
    TaskerTaskSucceeded(TaskerTaskDetail),
    /// Triggered when a task fails.
    TaskerTaskFailed(TaskerTaskDetail),

    // --- Node Pipeline Events ---
    /// Triggered when a node in the pipeline starts execution.
    NodePipelineNodeStarting(NodePipelineNodeDetail),
    /// Triggered when a node in the pipeline completes successfully.
    NodePipelineNodeSucceeded(NodePipelineNodeDetail),
    /// Triggered when a node in the pipeline fails.
    NodePipelineNodeFailed(NodePipelineNodeDetail),

    // --- Node Recognition Events ---
    /// Triggered when image recognition begins for a node.
    NodeRecognitionStarting(NodeRecognitionDetail),
    /// Triggered when image recognition succeeds.
    NodeRecognitionSucceeded(NodeRecognitionDetail),
    /// Triggered when image recognition fails.
    NodeRecognitionFailed(NodeRecognitionDetail),

    // --- Node Action Events ---
    /// Triggered before a node action is executed.
    NodeActionStarting(NodeActionDetail),
    /// Triggered after a node action completes successfully.
    NodeActionSucceeded(NodeActionDetail),
    /// Triggered if a node action fails.
    NodeActionFailed(NodeActionDetail),

    // --- Node Next List Events ---
    /// Triggered when processing the "next" list for a node.
    NodeNextListStarting(NodeNextListDetail),
    /// Triggered when the "next" list processing succeeds.
    NodeNextListSucceeded(NodeNextListDetail),
    /// Triggered when the "next" list processing fails.
    NodeNextListFailed(NodeNextListDetail),

    // --- Trace Events ---
    /// Trace event: Recognition node starting.
    NodeRecognitionNodeStarting(NodePipelineNodeDetail),
    /// Trace event: Recognition node succeeded.
    NodeRecognitionNodeSucceeded(NodePipelineNodeDetail),
    /// Trace event: Recognition node failed.
    NodeRecognitionNodeFailed(NodePipelineNodeDetail),

    /// Trace event: Action node starting.
    NodeActionNodeStarting(NodePipelineNodeDetail),
    /// Trace event: Action node succeeded.
    NodeActionNodeSucceeded(NodePipelineNodeDetail),
    /// Trace event: Action node failed.
    NodeActionNodeFailed(NodePipelineNodeDetail),

    // --- Fallback ---
    /// Represents an unknown or unparsable event.
    ///
    /// This variant is used as a fallback when the event message is not recognized
    /// or if JSON deserialization fails.
    Unknown {
        /// The raw message string.
        msg: String,
        /// The raw JSON detail string.
        raw_json: String,
        err: Option<String>,
    },
}

impl MaaEvent {
    /// Parses a notification message and detail string into a `MaaEvent`.
    ///
    /// This function handles the conversion from the raw C-string values provided by the
    /// framework callback into safe, typed Rust structures.
    ///
    /// # Arguments
    /// * `msg` - The notification type identifier (e.g., "Resource.Loading.Starting").
    /// * `details` - The JSON string containing event details.
    pub fn from_json(msg: &str, details: &str) -> Self {
        match msg {
            // Resource
            msg::RESOURCE_LOADING_STARTING => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::ResourceLoadingStarting(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::RESOURCE_LOADING_SUCCEEDED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::ResourceLoadingSucceeded(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::RESOURCE_LOADING_FAILED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::ResourceLoadingFailed(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },

            // Controller
            msg::CONTROLLER_ACTION_STARTING => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::ControllerActionStarting(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::CONTROLLER_ACTION_SUCCEEDED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::ControllerActionSucceeded(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::CONTROLLER_ACTION_FAILED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::ControllerActionFailed(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },

            // Tasker
            msg::TASKER_TASK_STARTING => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::TaskerTaskStarting(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::TASKER_TASK_SUCCEEDED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::TaskerTaskSucceeded(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::TASKER_TASK_FAILED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::TaskerTaskFailed(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },

            // Node Pipeline
            msg::NODE_PIPELINE_NODE_STARTING => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodePipelineNodeStarting(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::NODE_PIPELINE_NODE_SUCCEEDED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodePipelineNodeSucceeded(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::NODE_PIPELINE_NODE_FAILED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodePipelineNodeFailed(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },

            // Node Recognition
            msg::NODE_RECOGNITION_STARTING => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeRecognitionStarting(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::NODE_RECOGNITION_SUCCEEDED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeRecognitionSucceeded(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::NODE_RECOGNITION_FAILED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeRecognitionFailed(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },

            // Node Action
            msg::NODE_ACTION_STARTING => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeActionStarting(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::NODE_ACTION_SUCCEEDED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeActionSucceeded(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::NODE_ACTION_FAILED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeActionFailed(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },

            // Node Next List
            msg::NODE_NEXT_LIST_STARTING => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeNextListStarting(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::NODE_NEXT_LIST_SUCCEEDED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeNextListSucceeded(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::NODE_NEXT_LIST_FAILED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeNextListFailed(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },

            // Node Recognition Node
            msg::NODE_RECOGNITION_NODE_STARTING => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeRecognitionNodeStarting(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::NODE_RECOGNITION_NODE_SUCCEEDED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeRecognitionNodeSucceeded(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::NODE_RECOGNITION_NODE_FAILED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeRecognitionNodeFailed(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },

            // Node Action Node
            msg::NODE_ACTION_NODE_STARTING => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeActionNodeStarting(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::NODE_ACTION_NODE_SUCCEEDED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeActionNodeSucceeded(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },
            msg::NODE_ACTION_NODE_FAILED => match serde_json::from_str(details) {
                Ok(d) => MaaEvent::NodeActionNodeFailed(d),
                Err(e) => MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                },
            },

            _ => MaaEvent::Unknown {
                msg: msg.to_string(),
                raw_json: details.to_string(),
                err: None,
            },
        }
    }
}

// === Tests ===

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_context_event_parsing_logic() {
        let msg_reco = "Node.Recognition.Succeeded";
        let detail_reco =
            json!({ "task_id": 1, "reco_id": 100, "name": "R", "focus": null }).to_string();

        if let Some(ContextEvent::NodeRecognition(t, _)) =
            ContextEvent::from_notification(msg_reco, &detail_reco)
        {
            assert_eq!(t, NotificationType::Succeeded);
        } else {
            panic!("Node.Recognition parse failed");
        }

        let msg_node = "Node.RecognitionNode.Starting";
        let detail_node =
            json!({ "task_id": 1, "node_id": 200, "name": "N", "focus": null }).to_string();

        if let Some(ContextEvent::NodeRecognitionNode(t, _)) =
            ContextEvent::from_notification(msg_node, &detail_node)
        {
            assert_eq!(t, NotificationType::Starting);
        } else {
            panic!("Node.RecognitionNode parse failed");
        }
    }
}

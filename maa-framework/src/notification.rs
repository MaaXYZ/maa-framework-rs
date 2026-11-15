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
    /// Operation was stopped
    Cancelled,
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

    /// Check if this is a cancelled notification.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }

    /// Check if the operation is complete (succeeded, failed or cancelled).
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
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
        } else if s.ends_with(".Cancelled") {
            NotificationType::Cancelled
        } else {
            NotificationType::Unknown
        }
    }
}

// === Event Detail Structures ===

/// Resource loading event detail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceLoadingDetail {
    /// Resource ID
    pub res_id: i64,
    /// Resource hash
    pub hash: String,
    /// Path being loaded
    pub path: String,
    /// Load status, present in terminal notifications only
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<crate::common::MaaStatus>,
    /// Load diagnostics, present in terminal notifications only
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<ResourceLoadError>>,
}

/// A resource loading diagnostic.
///
/// Fields other than `code` are present only when their source is known.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceLoadError {
    /// Error code, e.g. `path_not_found` or `node_reference_not_found`
    pub code: String,
    /// Source file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Node name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    /// Source location as a JSON Pointer
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pointer: Option<String>,
    /// Field name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    /// Referenced target
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

/// Operation that created or directly contains an execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParentOperation {
    /// `task`, `node`, `list`, `recognition`, `action` or `wait_freezes`
    pub kind: String,
    /// Execution ID of that operation
    pub id: i64,
}

/// Common runtime fields of context notifications; absent before MaaFramework 6.0.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RuntimeFields {
    /// Context ID; clones receive new IDs
    #[serde(default)]
    pub context_id: Option<i64>,
    /// Parent task ID, `0` for a root task
    #[serde(default)]
    pub parent_task_id: Option<i64>,
    /// Operation that created the task, `None` for a root task
    #[serde(default)]
    pub task_parent: Option<ParentOperation>,
    /// Direct parent operation
    #[serde(default)]
    pub parent: Option<ParentOperation>,
    /// Termination reason, e.g. `completed`, `matched` or `cancelled`
    #[serde(default)]
    pub reason: Option<String>,
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
    /// Controller information (type, constructor parameters, etc.)
    #[serde(default)]
    pub info: Value,
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
    /// Termination reason in terminal notifications; a stopped task may succeed with `cancelled`
    #[serde(default)]
    pub reason: Option<String>,
}

/// Nested task event detail (`Context.Task.*`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextTaskDetail {
    /// Nested task ID
    pub task_id: i64,
    /// Entry node name
    pub entry: String,
    /// Common runtime fields
    #[serde(flatten)]
    pub runtime: RuntimeFields,
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
    /// ID of this candidate scan
    #[serde(default)]
    pub list_id: Option<i64>,
    /// Current pipeline node ID
    #[serde(default)]
    pub node_id: Option<i64>,
    /// Scan round, starting at 1 for each pipeline node
    #[serde(default)]
    pub round: Option<i64>,
    /// `entry`, `next` or `on_error`
    #[serde(default)]
    pub list_type: Option<String>,
    /// Zero-based index of the selected item, on success only
    #[serde(default)]
    pub selected_index: Option<i64>,
    /// Recognition ID of the selected item, on success only
    #[serde(default)]
    pub reco_id: Option<i64>,
    /// Common runtime fields
    #[serde(flatten)]
    pub runtime: RuntimeFields,
}

/// Node wait-freezes event detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeWaitFreezesDetail {
    /// Task ID
    pub task_id: i64,
    /// Wait-freezes ID
    pub wf_id: i64,
    /// Node name
    pub name: String,
    /// Phase name
    pub phase: String,
    /// ROI
    pub roi: crate::common::Rect,
    /// Wait-freezes parameters
    #[serde(default)]
    pub param: Value,
    /// Related recognition IDs
    #[serde(default)]
    pub reco_ids: Vec<i64>,
    /// Elapsed time in ms
    #[serde(default)]
    pub elapsed: Option<u64>,
    /// Focus configuration
    #[serde(default)]
    pub focus: Value,
    /// Common runtime fields
    #[serde(flatten)]
    pub runtime: RuntimeFields,
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
    /// Anchor name
    pub anchor: Option<String>,
    /// Effective pipeline, when [`crate::set_capture_config`] is enabled
    #[serde(default)]
    pub pipeline: Option<Value>,
    /// Common runtime fields
    #[serde(flatten)]
    pub runtime: RuntimeFields,
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
    /// Recognition ID used by this action
    #[serde(default)]
    pub reco_id: Option<i64>,
    /// Effective pipeline, when [`crate::set_capture_config`] is enabled
    #[serde(default)]
    pub pipeline: Option<Value>,
    /// Common runtime fields
    #[serde(flatten)]
    pub runtime: RuntimeFields,
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
    /// Common runtime fields
    #[serde(flatten)]
    pub runtime: RuntimeFields,
}

/// Scheduler transition event detail (`Node.Transition`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct NodeTransitionDetail {
    /// Task ID
    pub task_id: i64,
    /// Source node execution ID, `0` for the initial transition
    pub from_node_id: i64,
    /// Target node execution ID
    pub to_node_id: i64,
    /// Source node name
    pub from: String,
    /// Target node name
    pub to: String,
    /// `entry`, `next`, `on_error` or `jump_back`
    pub reason: String,
    /// Whether the selected node has the `jump_back` attribute
    pub jump_back: Option<bool>,
    /// Recognition ID of the selected node
    pub reco_id: Option<i64>,
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

    // Context nested task events
    pub const CONTEXT_TASK_STARTING: &str = "Context.Task.Starting";
    pub const CONTEXT_TASK_SUCCEEDED: &str = "Context.Task.Succeeded";
    pub const CONTEXT_TASK_FAILED: &str = "Context.Task.Failed";
    pub const CONTEXT_TASK_CANCELLED: &str = "Context.Task.Cancelled";

    // Scheduler transition event
    pub const NODE_TRANSITION: &str = "Node.Transition";

    // Node pipeline events
    pub const NODE_PIPELINE_NODE_STARTING: &str = "Node.PipelineNode.Starting";
    pub const NODE_PIPELINE_NODE_SUCCEEDED: &str = "Node.PipelineNode.Succeeded";
    pub const NODE_PIPELINE_NODE_FAILED: &str = "Node.PipelineNode.Failed";
    pub const NODE_PIPELINE_NODE_CANCELLED: &str = "Node.PipelineNode.Cancelled";

    // Node recognition events
    pub const NODE_RECOGNITION_STARTING: &str = "Node.Recognition.Starting";
    pub const NODE_RECOGNITION_SUCCEEDED: &str = "Node.Recognition.Succeeded";
    pub const NODE_RECOGNITION_FAILED: &str = "Node.Recognition.Failed";
    pub const NODE_RECOGNITION_CANCELLED: &str = "Node.Recognition.Cancelled";

    // Node action events
    pub const NODE_ACTION_STARTING: &str = "Node.Action.Starting";
    pub const NODE_ACTION_SUCCEEDED: &str = "Node.Action.Succeeded";
    pub const NODE_ACTION_FAILED: &str = "Node.Action.Failed";
    pub const NODE_ACTION_CANCELLED: &str = "Node.Action.Cancelled";

    // Node wait freezes events
    pub const NODE_WAIT_FREEZES_STARTING: &str = "Node.WaitFreezes.Starting";
    pub const NODE_WAIT_FREEZES_SUCCEEDED: &str = "Node.WaitFreezes.Succeeded";
    pub const NODE_WAIT_FREEZES_FAILED: &str = "Node.WaitFreezes.Failed";
    pub const NODE_WAIT_FREEZES_CANCELLED: &str = "Node.WaitFreezes.Cancelled";

    // Node next list events
    pub const NODE_NEXT_LIST_STARTING: &str = "Node.NextList.Starting";
    pub const NODE_NEXT_LIST_SUCCEEDED: &str = "Node.NextList.Succeeded";
    pub const NODE_NEXT_LIST_FAILED: &str = "Node.NextList.Failed";
    pub const NODE_NEXT_LIST_CANCELLED: &str = "Node.NextList.Cancelled";

    // Node recognition node trace events
    pub const NODE_RECOGNITION_NODE_STARTING: &str = "Node.RecognitionNode.Starting";
    pub const NODE_RECOGNITION_NODE_SUCCEEDED: &str = "Node.RecognitionNode.Succeeded";
    pub const NODE_RECOGNITION_NODE_FAILED: &str = "Node.RecognitionNode.Failed";
    pub const NODE_RECOGNITION_NODE_CANCELLED: &str = "Node.RecognitionNode.Cancelled";

    // Node action node trace events
    pub const NODE_ACTION_NODE_STARTING: &str = "Node.ActionNode.Starting";
    pub const NODE_ACTION_NODE_SUCCEEDED: &str = "Node.ActionNode.Succeeded";
    pub const NODE_ACTION_NODE_FAILED: &str = "Node.ActionNode.Failed";
    pub const NODE_ACTION_NODE_CANCELLED: &str = "Node.ActionNode.Cancelled";
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

/// Parse node wait-freezes event detail from JSON.
pub fn parse_node_wait_freezes(details: &str) -> Option<NodeWaitFreezesDetail> {
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
    NodeWaitFreezes(NotificationType, NodeWaitFreezesDetail),
    NodePipelineNode(NotificationType, NodePipelineNodeDetail),
    NodeRecognitionNode(NotificationType, NodePipelineNodeDetail),
    NodeActionNode(NotificationType, NodePipelineNodeDetail),
    ContextTask(NotificationType, ContextTaskDetail),
    NodeTransition(NodeTransitionDetail),
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

        if msg.starts_with("Node.WaitFreezes") {
            let detail = parse_node_wait_freezes(details)?;
            return Some(ContextEvent::NodeWaitFreezes(noti_type, detail));
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

        if msg.starts_with("Context.Task.") {
            let detail = serde_json::from_str(details).ok()?;
            return Some(ContextEvent::ContextTask(noti_type, detail));
        }

        if msg == msg::NODE_TRANSITION {
            let detail = serde_json::from_str(details).ok()?;
            return Some(ContextEvent::NodeTransition(detail));
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

    /// Called when a node wait-freezes event occurs.
    fn on_node_wait_freezes(&self, _noti_type: NotificationType, _detail: NodeWaitFreezesDetail) {}

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

    /// Called when a nested task event occurs.
    fn on_context_task(&self, _noti_type: NotificationType, _detail: ContextTaskDetail) {}

    /// Called after the selected node finishes and the scheduler moves on.
    fn on_node_transition(&self, _detail: NodeTransitionDetail) {}

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

    // --- Context Task Events ---
    /// Triggered when a nested task started by `Context::run_task` begins.
    ContextTaskStarting(ContextTaskDetail),
    /// Triggered when a nested task succeeds.
    ContextTaskSucceeded(ContextTaskDetail),
    /// Triggered when a nested task fails.
    ContextTaskFailed(ContextTaskDetail),
    /// Triggered when a nested task is stopped.
    ContextTaskCancelled(ContextTaskDetail),

    // --- Node Pipeline Events ---
    /// Triggered when a node in the pipeline starts execution.
    NodePipelineNodeStarting(NodePipelineNodeDetail),
    /// Triggered when a node in the pipeline completes successfully.
    NodePipelineNodeSucceeded(NodePipelineNodeDetail),
    /// Triggered when a node in the pipeline fails.
    NodePipelineNodeFailed(NodePipelineNodeDetail),
    /// Triggered when a node in the pipeline is stopped.
    NodePipelineNodeCancelled(NodePipelineNodeDetail),

    // --- Node Recognition Events ---
    /// Triggered when image recognition begins for a node.
    NodeRecognitionStarting(NodeRecognitionDetail),
    /// Triggered when image recognition succeeds.
    NodeRecognitionSucceeded(NodeRecognitionDetail),
    /// Triggered when image recognition fails.
    NodeRecognitionFailed(NodeRecognitionDetail),
    /// Triggered when image recognition is stopped.
    NodeRecognitionCancelled(NodeRecognitionDetail),

    // --- Node Action Events ---
    /// Triggered before a node action is executed.
    NodeActionStarting(NodeActionDetail),
    /// Triggered after a node action completes successfully.
    NodeActionSucceeded(NodeActionDetail),
    /// Triggered if a node action fails.
    NodeActionFailed(NodeActionDetail),
    /// Triggered if a node action is stopped.
    NodeActionCancelled(NodeActionDetail),

    // --- Node Wait Freezes Events ---
    /// Triggered when wait-freezes processing starts.
    NodeWaitFreezesStarting(NodeWaitFreezesDetail),
    /// Triggered when wait-freezes processing succeeds.
    NodeWaitFreezesSucceeded(NodeWaitFreezesDetail),
    /// Triggered when wait-freezes processing fails.
    NodeWaitFreezesFailed(NodeWaitFreezesDetail),
    /// Triggered when wait-freezes processing is stopped.
    NodeWaitFreezesCancelled(NodeWaitFreezesDetail),

    // --- Node Next List Events ---
    /// Triggered when processing the "next" list for a node.
    NodeNextListStarting(NodeNextListDetail),
    /// Triggered when the "next" list processing succeeds.
    NodeNextListSucceeded(NodeNextListDetail),
    /// Triggered when no node is selected in this round.
    NodeNextListFailed(NodeNextListDetail),
    /// Triggered when the "next" list processing is stopped.
    NodeNextListCancelled(NodeNextListDetail),

    // --- Trace Events ---
    /// Trace event: Recognition node starting.
    NodeRecognitionNodeStarting(NodePipelineNodeDetail),
    /// Trace event: Recognition node succeeded.
    NodeRecognitionNodeSucceeded(NodePipelineNodeDetail),
    /// Trace event: Recognition node failed.
    NodeRecognitionNodeFailed(NodePipelineNodeDetail),
    /// Trace event: Recognition node stopped.
    NodeRecognitionNodeCancelled(NodePipelineNodeDetail),

    /// Trace event: Action node starting.
    NodeActionNodeStarting(NodePipelineNodeDetail),
    /// Trace event: Action node succeeded.
    NodeActionNodeSucceeded(NodePipelineNodeDetail),
    /// Trace event: Action node failed.
    NodeActionNodeFailed(NodePipelineNodeDetail),
    /// Trace event: Action node stopped.
    NodeActionNodeCancelled(NodePipelineNodeDetail),

    /// Triggered after the selected node finishes and the scheduler moves on.
    NodeTransition(NodeTransitionDetail),

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
        fn parse<T: serde::de::DeserializeOwned>(
            msg: &str,
            details: &str,
            variant: fn(T) -> MaaEvent,
        ) -> MaaEvent {
            serde_json::from_str(details)
                .map(variant)
                .unwrap_or_else(|e| MaaEvent::Unknown {
                    msg: msg.to_string(),
                    raw_json: details.to_string(),
                    err: Some(e.to_string()),
                })
        }

        use MaaEvent as E;
        match msg {
            msg::RESOURCE_LOADING_STARTING => parse(msg, details, E::ResourceLoadingStarting),
            msg::RESOURCE_LOADING_SUCCEEDED => parse(msg, details, E::ResourceLoadingSucceeded),
            msg::RESOURCE_LOADING_FAILED => parse(msg, details, E::ResourceLoadingFailed),

            msg::CONTROLLER_ACTION_STARTING => parse(msg, details, E::ControllerActionStarting),
            msg::CONTROLLER_ACTION_SUCCEEDED => parse(msg, details, E::ControllerActionSucceeded),
            msg::CONTROLLER_ACTION_FAILED => parse(msg, details, E::ControllerActionFailed),

            msg::TASKER_TASK_STARTING => parse(msg, details, E::TaskerTaskStarting),
            msg::TASKER_TASK_SUCCEEDED => parse(msg, details, E::TaskerTaskSucceeded),
            msg::TASKER_TASK_FAILED => parse(msg, details, E::TaskerTaskFailed),

            msg::CONTEXT_TASK_STARTING => parse(msg, details, E::ContextTaskStarting),
            msg::CONTEXT_TASK_SUCCEEDED => parse(msg, details, E::ContextTaskSucceeded),
            msg::CONTEXT_TASK_FAILED => parse(msg, details, E::ContextTaskFailed),
            msg::CONTEXT_TASK_CANCELLED => parse(msg, details, E::ContextTaskCancelled),

            msg::NODE_PIPELINE_NODE_STARTING => parse(msg, details, E::NodePipelineNodeStarting),
            msg::NODE_PIPELINE_NODE_SUCCEEDED => parse(msg, details, E::NodePipelineNodeSucceeded),
            msg::NODE_PIPELINE_NODE_FAILED => parse(msg, details, E::NodePipelineNodeFailed),
            msg::NODE_PIPELINE_NODE_CANCELLED => parse(msg, details, E::NodePipelineNodeCancelled),

            msg::NODE_RECOGNITION_STARTING => parse(msg, details, E::NodeRecognitionStarting),
            msg::NODE_RECOGNITION_SUCCEEDED => parse(msg, details, E::NodeRecognitionSucceeded),
            msg::NODE_RECOGNITION_FAILED => parse(msg, details, E::NodeRecognitionFailed),
            msg::NODE_RECOGNITION_CANCELLED => parse(msg, details, E::NodeRecognitionCancelled),

            msg::NODE_ACTION_STARTING => parse(msg, details, E::NodeActionStarting),
            msg::NODE_ACTION_SUCCEEDED => parse(msg, details, E::NodeActionSucceeded),
            msg::NODE_ACTION_FAILED => parse(msg, details, E::NodeActionFailed),
            msg::NODE_ACTION_CANCELLED => parse(msg, details, E::NodeActionCancelled),

            msg::NODE_WAIT_FREEZES_STARTING => parse(msg, details, E::NodeWaitFreezesStarting),
            msg::NODE_WAIT_FREEZES_SUCCEEDED => parse(msg, details, E::NodeWaitFreezesSucceeded),
            msg::NODE_WAIT_FREEZES_FAILED => parse(msg, details, E::NodeWaitFreezesFailed),
            msg::NODE_WAIT_FREEZES_CANCELLED => parse(msg, details, E::NodeWaitFreezesCancelled),

            msg::NODE_NEXT_LIST_STARTING => parse(msg, details, E::NodeNextListStarting),
            msg::NODE_NEXT_LIST_SUCCEEDED => parse(msg, details, E::NodeNextListSucceeded),
            msg::NODE_NEXT_LIST_FAILED => parse(msg, details, E::NodeNextListFailed),
            msg::NODE_NEXT_LIST_CANCELLED => parse(msg, details, E::NodeNextListCancelled),

            msg::NODE_RECOGNITION_NODE_STARTING => {
                parse(msg, details, E::NodeRecognitionNodeStarting)
            }
            msg::NODE_RECOGNITION_NODE_SUCCEEDED => {
                parse(msg, details, E::NodeRecognitionNodeSucceeded)
            }
            msg::NODE_RECOGNITION_NODE_FAILED => parse(msg, details, E::NodeRecognitionNodeFailed),
            msg::NODE_RECOGNITION_NODE_CANCELLED => {
                parse(msg, details, E::NodeRecognitionNodeCancelled)
            }

            msg::NODE_ACTION_NODE_STARTING => parse(msg, details, E::NodeActionNodeStarting),
            msg::NODE_ACTION_NODE_SUCCEEDED => parse(msg, details, E::NodeActionNodeSucceeded),
            msg::NODE_ACTION_NODE_FAILED => parse(msg, details, E::NodeActionNodeFailed),
            msg::NODE_ACTION_NODE_CANCELLED => parse(msg, details, E::NodeActionNodeCancelled),

            msg::NODE_TRANSITION => parse(msg, details, E::NodeTransition),

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
        let msg_wf = "Node.WaitFreezes.Succeeded";
        let detail_wf = json!({
            "task_id": 1,
            "wf_id": 10,
            "name": "WF",
            "phase": "pre",
            "roi": [1, 2, 3, 4],
            "param": {},
            "reco_ids": [100, 101],
            "elapsed": 233,
            "focus": null
        })
        .to_string();

        if let Some(ContextEvent::NodeWaitFreezes(t, d)) =
            ContextEvent::from_notification(msg_wf, &detail_wf)
        {
            assert_eq!(t, NotificationType::Succeeded);
            assert_eq!(d.wf_id, 10);
            assert_eq!(d.reco_ids, vec![100, 101]);
            assert_eq!(d.elapsed, Some(233));
        } else {
            panic!("Node.WaitFreezes parse failed");
        }
        let msg_reco = "Node.Recognition.Succeeded";
        let detail_reco =
            json!({ "task_id": 1, "reco_id": 100, "name": "R", "focus": null, "anchor": "A1" })
                .to_string();

        if let Some(ContextEvent::NodeRecognition(t, d)) =
            ContextEvent::from_notification(msg_reco, &detail_reco)
        {
            assert_eq!(t, NotificationType::Succeeded);
            assert_eq!(d.anchor.as_deref(), Some("A1"));
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
    #[test]
    fn test_runtime_event_parsing() {
        let mut payload = json!({
            "task_id": 10,
            "name": "A",
            "list": [{"name": "B", "jump_back": false, "anchor": false}],
            "focus": null
        });
        match ContextEvent::from_notification("Node.NextList.Succeeded", &payload.to_string()) {
            Some(ContextEvent::NodeNextList(_, d)) => {
                assert_eq!(d.list_id, None);
                assert_eq!(d.runtime, RuntimeFields::default());
            }
            other => panic!("Node.NextList parse failed: {other:?}"),
        }

        let extra = json!({
            "context_id": 12, "parent_task_id": 9, "parent": {"kind": "node", "id": 25},
            "list_id": 50, "node_id": 25, "round": 2, "list_type": "on_error", "reason": "cancelled"
        });
        payload
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        match MaaEvent::from_json("Node.NextList.Cancelled", &payload.to_string()) {
            MaaEvent::NodeNextListCancelled(d) => {
                assert_eq!(
                    (
                        d.runtime.context_id,
                        d.runtime.parent_task_id,
                        d.list_id,
                        d.round
                    ),
                    (Some(12), Some(9), Some(50), Some(2))
                );
                assert_eq!(d.list_type.as_deref(), Some("on_error"));
                assert_eq!(
                    d.runtime.parent,
                    Some(ParentOperation {
                        kind: "node".to_string(),
                        id: 25
                    })
                );
                assert_eq!(d.runtime.reason.as_deref(), Some("cancelled"));
            }
            other => panic!("Node.NextList.Cancelled parse failed: {other:?}"),
        }
        assert_eq!(
            parse_type("Node.NextList.Cancelled"),
            NotificationType::Cancelled
        );

        let child = json!({"task_id": 11, "entry": "Child", "parent_task_id": 10}).to_string();
        match ContextEvent::from_notification("Context.Task.Starting", &child) {
            Some(ContextEvent::ContextTask(NotificationType::Starting, d)) => {
                assert_eq!(d.runtime.parent_task_id, Some(10))
            }
            other => panic!("Context.Task parse failed: {other:?}"),
        }

        match MaaEvent::from_json("Node.Transition", r#"{"reason": "jump_back"}"#) {
            MaaEvent::NodeTransition(d) => assert_eq!(d.reason, "jump_back"),
            other => panic!("Node.Transition parse failed: {other:?}"),
        }
        assert_eq!(parse_type("Unknown.Cancelled"), NotificationType::Cancelled);
        assert!(NotificationType::Cancelled.is_complete());
    }
}

//! Pipeline configuration types for recognition and action definitions.

use serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;

// --- Custom Deserializers for Scalar/Array Polymorphism ---
// The C API may return either a scalar or an array for some fields.

/// Deserialize a value that can be either T or Vec<T> into Vec<T>
fn scalar_or_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: DeserializeOwned,
{
    let value = Value::deserialize(deserializer)?;

    // Try to parse as Vec<T> first
    if let Ok(vec) = serde_json::from_value::<Vec<T>>(value.clone()) {
        return Ok(vec);
    }

    // Fallback to T
    if let Ok(single) = serde_json::from_value::<T>(value) {
        return Ok(vec![single]);
    }

    Err(serde::de::Error::custom("Expected T or Vec<T>"))
}

// --- Common Types ---

/// Rectangle coordinates: (x, y, width, height).
pub type Rect = (i32, i32, i32, i32);
/// Region of interest: (x, y, width, height). Use [0, 0, 0, 0] for full screen.
pub type Roi = (i32, i32, i32, i32);

/// Target can be:
/// - true: recognized position
/// - "NodeName": position from previously executed node
/// - \[ x, y \]: point (2 elements)
/// - \[ x, y, w, h \]: area (4 elements)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Target {
    Bool(bool),
    Name(String),
    Point((i32, i32)),
    Rect(Rect),
}

impl Default for Target {
    fn default() -> Self {
        Target::Bool(true)
    }
}

/// Anchor configuration.
///
/// Can be:
/// - String: Set anchor to current node.
/// - List of strings: Set multiple anchors to current node.
/// - Map: Set anchors to specific nodes (or clear if empty).
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Anchor {
    Name(String),
    List(Vec<String>),
    Map(HashMap<String, String>),
}

impl Default for Anchor {
    fn default() -> Self {
        Anchor::List(Vec::new())
    }
}

// --- Node Attribute ---

/// Node attribute for specifying behavior in `next` and `on_error` lists.
///
/// Allows setting additional control parameters when referencing nodes.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct NodeAttr {
    /// Node name to reference.
    #[serde(default)]
    pub name: String,
    /// Whether to return to this node after the referenced node completes.
    #[serde(default)]
    pub jump_back: bool,
    /// Whether to use an anchor reference.
    #[serde(default)]
    pub anchor: bool,
}

// --- Wait Freezes ---

/// Configuration for waiting until the screen stops changing.
///
/// Used in `pre_wait_freezes`, `post_wait_freezes`, and `repeat_wait_freezes`
/// to wait for the screen to stabilize before/after actions.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WaitFreezes {
    /// Duration in milliseconds the screen must remain stable. Default: 1.
    #[serde(default = "default_wait_time")]
    pub time: i32,
    /// Target area to monitor for changes.
    #[serde(default)]
    pub target: Target,
    /// Offset applied to the target area.
    #[serde(default)]
    pub target_offset: Rect,
    /// Similarity threshold for detecting changes. Default: 0.95.
    #[serde(default = "default_wait_threshold")]
    pub threshold: f64,
    /// Comparison method (cv::TemplateMatchModes). Default: 5.
    #[serde(default = "default_wait_method")]
    pub method: i32,
    /// Minimum interval between checks in milliseconds. Default: 1000.
    #[serde(default = "default_rate_limit")]
    pub rate_limit: i32,
    /// Overall timeout in milliseconds. Default: 20000.
    #[serde(default = "default_timeout")]
    pub timeout: i32,
}

impl Default for WaitFreezes {
    fn default() -> Self {
        Self {
            time: default_wait_time(),
            target: Target::default(),
            target_offset: (0, 0, 0, 0),
            threshold: default_wait_threshold(),
            method: default_wait_method(),
            rate_limit: default_rate_limit(),
            timeout: default_timeout(),
        }
    }
}

// --- Recognition Enums ---

/// Recognition algorithm types.
///
/// Determines how the framework identifies targets on screen:
/// - [`DirectHit`] - No recognition, always matches
/// - [`TemplateMatch`] - Image template matching
/// - [`FeatureMatch`] - Feature-based matching (rotation/scale invariant)
/// - [`ColorMatch`] - Color-based matching
/// - [`OCR`] - Optical character recognition
/// - [`NeuralNetworkClassify`] - Deep learning classification
/// - [`NeuralNetworkDetect`] - Deep learning object detection
/// - [`And`] - Logical AND of multiple recognitions
/// - [`Or`] - Logical OR of multiple recognitions
/// - `Custom` - User-defined recognition
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "param")]
pub enum Recognition {
    DirectHit(DirectHit),
    TemplateMatch(TemplateMatch),
    FeatureMatch(FeatureMatch),
    ColorMatch(ColorMatch),
    OCR(OCR),
    NeuralNetworkClassify(NeuralNetworkClassify),
    NeuralNetworkDetect(NeuralNetworkDetect),
    And(And),
    Or(Or),
    Custom(CustomRecognition),
}

/// Reference to a recognition: either an inline definition or a node name.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum RecognitionRef {
    NodeName(String),
    Inline(Recognition),
}

// --- Specific Recognition Structs ---

/// Direct hit recognition - always matches without performing actual recognition.
///
/// Use when you want to execute an action without image matching.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DirectHit {
    /// Recognition region. Default: \\[0,0,0,0\\] (full screen).
    #[serde(default = "default_roi_zero")]
    pub roi: Target,
    /// Offset applied to the ROI.
    #[serde(default)]
    pub roi_offset: Rect,
}

/// Template matching recognition - finds images using OpenCV template matching.
///
/// The most common recognition method for "finding images" on screen.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TemplateMatch {
    /// Template image paths relative to `image` folder. Required.
    #[serde(deserialize_with = "scalar_or_vec")]
    pub template: Vec<String>,
    /// Recognition region. Default: \\[0,0,0,0\\] (full screen).
    #[serde(default = "default_roi_zero")]
    pub roi: Target,
    /// Offset applied to the ROI.
    #[serde(default)]
    pub roi_offset: Rect,
    /// Matching threshold(s). Default: [0.7].
    #[serde(default = "default_threshold", deserialize_with = "scalar_or_vec")]
    pub threshold: Vec<f64>,
    /// Result sorting: "Horizontal", "Vertical", "Score", "Random". Default: "Horizontal".
    #[serde(default = "default_order_by")]
    pub order_by: String,
    /// Which result to select (0-indexed, negative for reverse). Default: 0.
    #[serde(default)]
    pub index: i32,
    /// OpenCV matching method (cv::TemplateMatchModes). Default: 5 (TM_CCOEFF_NORMED).
    #[serde(default = "default_template_method")]
    pub method: i32,
    /// Use green (0,255,0) as mask color. Default: false.
    #[serde(default)]
    pub green_mask: bool,
}

/// Feature-based matching - scale and rotation invariant image matching.
///
/// More robust than template matching for detecting objects under transformation.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FeatureMatch {
    /// Template image paths relative to `image` folder. Required.
    #[serde(deserialize_with = "scalar_or_vec")]
    pub template: Vec<String>,
    /// Recognition region. Default: \\[0,0,0,0\\] (full screen).
    #[serde(default = "default_roi_zero")]
    pub roi: Target,
    /// Offset applied to the ROI.
    #[serde(default)]
    pub roi_offset: Rect,
    /// Feature detector: "SIFT", "KAZE", "AKAZE", "BRISK", "ORB". Default: "SIFT".
    #[serde(default = "default_detector")]
    pub detector: String,
    /// Result sorting method. Default: "Horizontal".
    #[serde(default = "default_order_by")]
    pub order_by: String,
    /// Minimum feature point matches required. Default: 4.
    #[serde(default = "default_feature_count")]
    pub count: i32,
    /// Which result to select. Default: 0.
    #[serde(default)]
    pub index: i32,
    /// Use green (0,255,0) as mask color. Default: false.
    #[serde(default)]
    pub green_mask: bool,
    /// KNN distance ratio threshold [0-1.0]. Default: 0.6.
    #[serde(default = "default_feature_ratio")]
    pub ratio: f64,
}

/// Color matching recognition - finds regions by color range.
///
/// Matches pixels within specified color bounds.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ColorMatch {
    /// Lower color bounds. Required. Format depends on method.
    #[serde(deserialize_with = "scalar_or_vec")]
    pub lower: Vec<Vec<i32>>,
    /// Upper color bounds. Required. Format depends on method.
    #[serde(deserialize_with = "scalar_or_vec")]
    pub upper: Vec<Vec<i32>>,
    /// Recognition region. Default: \\[0,0,0,0\\] (full screen).
    #[serde(default = "default_roi_zero")]
    pub roi: Target,
    /// Offset applied to the ROI.
    #[serde(default)]
    pub roi_offset: Rect,
    /// Result sorting method. Default: "Horizontal".
    #[serde(default = "default_order_by")]
    pub order_by: String,
    /// Color conversion code (cv::ColorConversionCodes). Default: 4 (RGB).
    #[serde(default = "default_color_method")]
    pub method: i32,
    /// Minimum matching pixel count. Default: 1.
    #[serde(default = "default_count_one")]
    pub count: i32,
    /// Which result to select. Default: 0.
    #[serde(default)]
    pub index: i32,
    /// Only count connected pixels. Default: false.
    #[serde(default)]
    pub connected: bool,
}

/// Optical character recognition - finds and reads text.
///
/// Uses OCR model to detect and recognize text in the specified region.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct OCR {
    /// Expected text patterns (supports regex). Default: match all.
    #[serde(default, deserialize_with = "scalar_or_vec")]
    pub expected: Vec<String>,
    /// Recognition region. Default: \\[0,0,0,0\\] (full screen).
    #[serde(default = "default_roi_zero")]
    pub roi: Target,
    /// Offset applied to the ROI.
    #[serde(default)]
    pub roi_offset: Rect,
    /// Model confidence threshold. Default: 0.3.
    #[serde(default = "default_ocr_threshold")]
    pub threshold: f64,
    /// Text replacement pairs [[from, to], ...] for fixing OCR errors.
    #[serde(default)]
    pub replace: Vec<Vec<String>>,
    /// Result sorting method. Default: "Horizontal".
    #[serde(default = "default_order_by")]
    pub order_by: String,
    /// Which result to select. Default: 0.
    #[serde(default)]
    pub index: i32,
    /// Recognition only (skip detection, requires precise ROI). Default: false.
    #[serde(default)]
    pub only_rec: bool,
    /// Model folder path relative to `model/ocr`. Default: root.
    #[serde(default)]
    pub model: String,
}

/// Neural network classification - classifies fixed regions.
///
/// Uses ONNX model to classify images at fixed positions.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NeuralNetworkClassify {
    /// Model file path relative to `model/classify`. Required.
    pub model: String,
    /// Expected class indices to match. Default: match all.
    #[serde(default, deserialize_with = "scalar_or_vec")]
    pub expected: Vec<i32>,
    /// Recognition region. Default: \\[0,0,0,0\\] (full screen).
    #[serde(default = "default_roi_zero")]
    pub roi: Target,
    /// Offset applied to the ROI.
    #[serde(default)]
    pub roi_offset: Rect,
    /// Class labels for debugging. Default: "Unknown".
    #[serde(default)]
    pub labels: Vec<String>,
    /// Result sorting method. Default: "Horizontal".
    #[serde(default = "default_order_by")]
    pub order_by: String,
    /// Which result to select. Default: 0.
    #[serde(default)]
    pub index: i32,
}

/// Neural network detection - detects objects anywhere on screen.
///
/// Uses YOLO-style ONNX model to detect and locate objects.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NeuralNetworkDetect {
    /// Model file path relative to `model/detect`. Required.
    pub model: String,
    /// Expected class indices to match. Default: match all.
    #[serde(default, deserialize_with = "scalar_or_vec")]
    pub expected: Vec<i32>,
    /// Recognition region. Default: \\[0,0,0,0\\] (full screen).
    #[serde(default = "default_roi_zero")]
    pub roi: Target,
    /// Offset applied to the ROI.
    #[serde(default)]
    pub roi_offset: Rect,
    /// Class labels (auto-read from model metadata). Default: "Unknown".
    #[serde(default)]
    pub labels: Vec<String>,
    /// Confidence threshold(s). Default: [0.3].
    #[serde(
        default = "default_detect_threshold",
        deserialize_with = "scalar_or_vec"
    )]
    pub threshold: Vec<f64>,
    /// Result sorting method. Default: "Horizontal".
    #[serde(default = "default_order_by")]
    pub order_by: String,
    /// Which result to select. Default: 0.
    #[serde(default)]
    pub index: i32,
}

/// Custom recognition - uses user-registered recognition handler.
///
/// Invokes a handler registered via `MaaResourceRegisterCustomRecognition`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomRecognition {
    /// Handler name (as registered). Required.
    pub custom_recognition: String,
    /// Recognition region. Default: \\[0,0,0,0\\] (full screen).
    #[serde(default = "default_roi_zero")]
    pub roi: Target,
    /// Offset applied to the ROI.
    #[serde(default)]
    pub roi_offset: Rect,
    /// Custom parameters passed to the handler.
    #[serde(default)]
    pub custom_recognition_param: Value,
}

/// Logical AND recognition - all sub-recognitions must match.
///
/// Combines multiple recognitions; succeeds only when all match.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct And {
    /// Sub-recognition list. All must match. Required.
    #[serde(default)]
    pub all_of: Vec<RecognitionRef>,
    /// Which sub-recognition's bounding box to use. Default: 0.
    #[serde(default)]
    pub box_index: i32,
}

/// Logical OR recognition - first matching sub-recognition wins.
///
/// Combines multiple recognitions; succeeds when any one matches.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Or {
    /// Sub-recognition list. First match wins. Required.
    #[serde(default)]
    pub any_of: Vec<RecognitionRef>,
}

// --- Action Enums ---

/// Action types executed after successful recognition.
///
/// - [`DoNothing`] - No action
/// - [`Click`] - Tap/click
/// - [`LongPress`] - Long press
/// - [`Swipe`] - Linear swipe
/// - [`MultiSwipe`] - Multi-touch swipe
/// - Touch actions: `TouchDown`, `TouchMove`, [`TouchUp`]
/// - Key actions: `ClickKey`, [`LongPressKey`], `KeyDown`, `KeyUp`
/// - [`InputText`] - Text input
/// - App control: `StartApp`, `StopApp`
/// - [`StopTask`] - Stop current task
/// - [`Scroll`] - Mouse wheel scroll
/// - [`Command`] - Execute local command
/// - [`Shell`] - Execute ADB shell command
/// - `Custom` - User-defined action
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "param")]
pub enum Action {
    DoNothing(DoNothing),
    Click(Click),
    LongPress(LongPress),
    Swipe(Swipe),
    MultiSwipe(MultiSwipe),
    TouchDown(Touch),
    TouchMove(Touch),
    TouchUp(TouchUp),
    ClickKey(KeyList),
    LongPressKey(LongPressKey),
    KeyDown(SingleKey),
    KeyUp(SingleKey),
    InputText(InputText),
    StartApp(App),
    StopApp(App),
    StopTask(StopTask),
    Scroll(Scroll),
    Command(Command),
    Shell(Shell),
    Custom(CustomAction),
}

// --- Action Structs ---

/// Do nothing action.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct DoNothing {}

/// Stop current task chain action.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct StopTask {}

/// Click/tap action.
///
/// Performs a single tap at the target position.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Click {
    /// Click target position. Default: recognized position.
    #[serde(default)]
    pub target: Target,
    /// Offset applied to target.
    #[serde(default)]
    pub target_offset: Rect,
    /// Touch contact/button index. Default: 0.
    #[serde(default)]
    pub contact: i32,
    /// Touch pressure. Default: 1.
    #[serde(default = "default_pressure")]
    pub pressure: i32,
}

/// Long press action.
///
/// Performs a sustained press at the target position.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LongPress {
    /// Press target position. Default: recognized position.
    #[serde(default)]
    pub target: Target,
    /// Offset applied to target.
    #[serde(default)]
    pub target_offset: Rect,
    /// Press duration in milliseconds. Default: 1000.
    #[serde(default = "default_long_press_duration")]
    pub duration: i32,
    /// Touch contact/button index. Default: 0.
    #[serde(default)]
    pub contact: i32,
    /// Touch pressure. Default: 1.
    #[serde(default = "default_pressure")]
    pub pressure: i32,
}

/// Linear swipe action.
///
/// Swipes from begin to end position(s). Supports waypoints.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Swipe {
    /// Start time offset in ms (for MultiSwipe). Default: 0.
    #[serde(default)]
    pub starting: i32,
    /// Swipe start position. Default: recognized position.
    #[serde(default)]
    pub begin: Target,
    /// Offset applied to begin.
    #[serde(default)]
    pub begin_offset: Rect,
    /// Swipe end position(s). Supports waypoints. Default: recognized position.
    #[serde(
        default = "default_target_list_true",
        deserialize_with = "scalar_or_vec"
    )]
    pub end: Vec<Target>,
    /// Offset(s) applied to end.
    #[serde(default = "default_rect_list_zero", deserialize_with = "scalar_or_vec")]
    pub end_offset: Vec<Rect>,
    /// Hold time at end position(s) in ms. Default: \\[0\\].
    #[serde(default = "default_i32_list_zero", deserialize_with = "scalar_or_vec")]
    pub end_hold: Vec<i32>,
    /// Duration(s) in milliseconds. Default: \\[200\\].
    #[serde(default = "default_duration_list", deserialize_with = "scalar_or_vec")]
    pub duration: Vec<i32>,
    /// Hover only (no press). Default: false.
    #[serde(default)]
    pub only_hover: bool,
    /// Touch contact/button index. Default: 0.
    #[serde(default)]
    pub contact: i32,
    /// Touch pressure. Default: 1.
    #[serde(default = "default_pressure")]
    pub pressure: i32,
}

/// Multi-finger swipe action.
///
/// Performs multiple simultaneous swipes (e.g., pinch gestures).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MultiSwipe {
    /// List of swipe configurations.
    #[serde(default)]
    pub swipes: Vec<Swipe>,
}

/// Touch down/move action - initiates or moves a touch point.
///
/// Used for custom touch sequences. Pair with TouchUp to complete.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Touch {
    /// Touch contact index. Default: 0.
    #[serde(default)]
    pub contact: i32,
    /// Touch target position. Default: recognized position.
    #[serde(default)]
    pub target: Target,
    /// Offset applied to target.
    #[serde(default)]
    pub target_offset: Rect,
    /// Touch pressure. Default: 0.
    #[serde(default)]
    pub pressure: i32,
}

/// Touch up action - releases a touch point.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TouchUp {
    /// Touch contact index to release. Default: 0.
    #[serde(default)]
    pub contact: i32,
}

/// Long press key action.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LongPressKey {
    /// Virtual key code(s) to press. Required.
    #[serde(deserialize_with = "scalar_or_vec")]
    pub key: Vec<i32>,
    /// Press duration in milliseconds. Default: 1000.
    #[serde(default = "default_long_press_duration")]
    pub duration: i32,
}

/// Click key action - single key press.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyList {
    /// Virtual key code(s) to click. Required.
    #[serde(deserialize_with = "scalar_or_vec")]
    pub key: Vec<i32>,
}

/// Single key action - for KeyDown/KeyUp.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SingleKey {
    /// Virtual key code. Required.
    pub key: i32,
}

/// Text input action.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InputText {
    /// Text to input (ASCII recommended). Required.
    pub input_text: String,
}

/// App control action - for StartApp/StopApp.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct App {
    /// Package name or activity (e.g., "com.example.app"). Required.
    pub package: String,
}

/// Mouse scroll action (Win32 only).
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Scroll {
    /// Scroll target position. Default: recognized position.
    #[serde(default)]
    pub target: Target,
    /// Offset applied to target.
    #[serde(default)]
    pub target_offset: Rect,
    /// Horizontal scroll delta. Default: 0.
    #[serde(default)]
    pub dx: i32,
    /// Vertical scroll delta. Default: 0.
    #[serde(default)]
    pub dy: i32,
}

/// Execute local command action.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Command {
    /// Program path to execute. Required.
    pub exec: String,
    /// Command arguments. Supports runtime placeholders.
    #[serde(default)]
    pub args: Vec<String>,
    /// Run in background (don't wait). Default: false.
    #[serde(default)]
    pub detach: bool,
}

/// Execute ADB shell command action.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Shell {
    /// Shell command to execute. Required.
    pub cmd: String,
    /// Command timeout in milliseconds. Default: 20000.
    #[serde(default = "default_timeout")]
    pub timeout: i32,
}

/// Custom action - uses user-registered action handler.
///
/// Invokes a handler registered via `MaaResourceRegisterCustomAction`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomAction {
    /// Handler name (as registered). Required.
    pub custom_action: String,
    /// Target position passed to handler. Default: recognized position.
    #[serde(default)]
    pub target: Target,
    /// Custom parameters passed to the handler.
    #[serde(default)]
    pub custom_action_param: Value,
    /// Offset applied to target.
    #[serde(default)]
    pub target_offset: Rect,
}

// --- Pipeline Data ---

/// Complete pipeline node configuration.
///
/// Defines a node's recognition, action, and flow control parameters.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PipelineData {
    /// Recognition algorithm configuration.
    pub recognition: Recognition,
    /// Action to execute on match.
    pub action: Action,
    /// Next nodes to check after action. Default: [].
    #[serde(default)]
    pub next: Vec<NodeAttr>,
    /// Recognition rate limit in ms. Default: 1000.
    #[serde(default = "default_rate_limit")]
    pub rate_limit: i32,
    /// Overall timeout in ms. Default: 20000.
    #[serde(default = "default_timeout")]
    pub timeout: i32,
    /// Nodes to check on timeout/error. Default: [].
    #[serde(default)]
    pub on_error: Vec<NodeAttr>,
    /// Anchor names for this node. Default: [].
    #[serde(default)]
    pub anchor: Anchor,
    /// Invert recognition result. Default: false.
    #[serde(default)]
    pub inverse: bool,
    /// Enable this node. Default: true.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Delay before action in ms. Default: 200.
    #[serde(default = "default_pre_delay")]
    pub pre_delay: i32,
    /// Delay after action in ms. Default: 200.
    #[serde(default = "default_post_delay")]
    pub post_delay: i32,
    /// Wait for screen stability before action.
    #[serde(default)]
    pub pre_wait_freezes: Option<WaitFreezes>,
    /// Wait for screen stability after action.
    #[serde(default)]
    pub post_wait_freezes: Option<WaitFreezes>,
    /// Action repeat count. Default: 1.
    #[serde(default = "default_repeat")]
    pub repeat: i32,
    /// Delay between repeats in ms. Default: 0.
    #[serde(default)]
    pub repeat_delay: i32,
    /// Wait for stability between repeats.
    #[serde(default)]
    pub repeat_wait_freezes: Option<WaitFreezes>,
    /// Maximum successful hits. Default: UINT_MAX.
    #[serde(default = "default_max_hit")]
    pub max_hit: u32,
    /// Focus flag for extra callbacks. Default: null.
    #[serde(default)]
    pub focus: Option<Value>,
    /// Attached custom data (merged with defaults).
    #[serde(default)]
    pub attach: Option<Value>,
}

// --- Defaults Helper Functions ---

fn default_wait_time() -> i32 {
    1
}
fn default_wait_threshold() -> f64 {
    0.95
}
fn default_wait_method() -> i32 {
    5
}
fn default_rate_limit() -> i32 {
    1000
}
fn default_timeout() -> i32 {
    20000
}
fn default_threshold() -> Vec<f64> {
    vec![0.7]
}
fn default_order_by() -> String {
    "Horizontal".to_string()
}
fn default_template_method() -> i32 {
    5
}
fn default_detector() -> String {
    "SIFT".to_string()
}
fn default_feature_count() -> i32 {
    4
}
fn default_feature_ratio() -> f64 {
    0.6
}
fn default_color_method() -> i32 {
    4
} // RGB
fn default_count_one() -> i32 {
    1
}
fn default_ocr_threshold() -> f64 {
    0.3
}
fn default_detect_threshold() -> Vec<f64> {
    vec![0.3]
}
fn default_pressure() -> i32 {
    1
}
fn default_long_press_duration() -> i32 {
    1000
}
fn default_target_list_true() -> Vec<Target> {
    vec![Target::Bool(true)]
}
fn default_rect_list_zero() -> Vec<Rect> {
    vec![(0, 0, 0, 0)]
}
fn default_i32_list_zero() -> Vec<i32> {
    vec![0]
}
fn default_duration_list() -> Vec<i32> {
    vec![200]
}
fn default_enabled() -> bool {
    true
}
fn default_pre_delay() -> i32 {
    200
}
fn default_post_delay() -> i32 {
    200
}
fn default_repeat() -> i32 {
    1
}
fn default_max_hit() -> u32 {
    u32::MAX
}
fn default_roi_zero() -> Target {
    Target::Rect((0, 0, 0, 0))
}

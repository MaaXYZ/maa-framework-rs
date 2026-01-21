//! Comprehensive binding tests matching Python binding_test.py
//!
//! Test coverage:
//! - Resource: loading, inference settings, custom registration, event sinks
//! - Controller: DbgController, connection, input operations, screenshots
//! - Tasker: task execution, status queries, detail retrieval, event sinks
//! - CustomController: custom controller implementation
//! - Toolkit: device discovery

mod common;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use maa_framework::context::Context;
use maa_framework::controller::Controller;
use maa_framework::custom::{CustomAction, CustomRecognition};
use maa_framework::custom_controller::CustomControllerCallback;
use maa_framework::resource::Resource;
use maa_framework::tasker::Tasker;
use maa_framework::toolkit::Toolkit;
use maa_framework::{self, sys};

use common::{get_test_resources_dir, init_test_env};

// ============================================================================
// Custom Recognition/Action for testing
// ============================================================================

static ANALYZED: AtomicBool = AtomicBool::new(false);
static RUNNED: AtomicBool = AtomicBool::new(false);

struct MyRecognition;

impl CustomRecognition for MyRecognition {
    fn analyze(
        &self,
        context: &Context,
        _task_id: sys::MaaTaskId,
        node_name: &str,
        custom_recognition_name: &str,
        _custom_recognition_param: &str,
        _image: *const sys::MaaImageBuffer,
        _roi: &sys::MaaRect,
    ) -> Option<(sys::MaaRect, String)> {
        println!(
            "on MyRecognition.analyze, context: {:?}, node: {}, reco: {}",
            context.raw(),
            node_name,
            custom_recognition_name
        );

        // Verify methods related to task context, including running tasks, recognition, and actions.
        // ================================================================

        // Verify run_task functionality
        let ppover = r#"{
            "ColorMatch": {
                "recognition": "ColorMatch",
                "lower": [100, 100, 100],
                "upper": [255, 255, 255],
                "action": "Click"
            }
        }"#;
        let run_result = context.run_task("ColorMatch", ppover);
        println!("  run_task result: {:?}", run_result);

        // Verify direct action and recognition execution
        let rect = maa_framework::common::Rect {
            x: 114,
            y: 514,
            width: 191,
            height: 810,
        };
        let action_detail = context.run_action("ColorMatch", ppover, &rect, "RunAction Detail");
        println!("  run_action result: {:?}", action_detail);

        // Use a dummy buffer for testing API calls (content irrelevant for this test)
        let dummy_img = maa_framework::buffer::MaaImageBuffer::new();
        if let Ok(img) = dummy_img {
            let reco_detail = context.run_recognition("ColorMatch", ppover, &img);
            println!("  run_recognition result: {:?}", reco_detail);

            // Verify run_recognition_direct
            let reco_direct =
                context.run_recognition_direct("OCR", "{\"expected\": \"test\"}", &img);
            println!("  run_recognition_direct result: {:?}", reco_direct);

            // Verify run_action_direct
            let action_direct = context.run_action_direct("Click", "{}", &rect, "");
            println!("  run_action_direct result: {:?}", action_direct);
        }

        // Verify context cloning and dynamic pipeline overrides
        if let Ok(new_ctx) = context.clone_context() {
            println!("  clone_context: OK");

            // Verify pipeline overriding within cloned context
            let override_result = new_ctx.override_pipeline(r#"{"TaskA": {}, "TaskB": {}}"#);
            println!("  override_pipeline result: {:?}", override_result);

            // Verify next-node overriding within cloned context
            let override_next = new_ctx.override_next(node_name, &["TaskA", "TaskB"]);
            println!("  override_next result: {:?}", override_next);

            // Verify retrieval of node data from context
            let node_data = new_ctx.get_node_data(node_name);
            println!(
                "  get_node_data({}) is_some: {}",
                node_name,
                node_data.as_ref().map_or(false, |o| o.is_some())
            );

            // Verify anchor setting and retrieval
            let set_result = new_ctx.set_anchor("test_anchor", "TaskA");
            println!("  set_anchor result: {:?}", set_result);

            let anchor = new_ctx.get_anchor("test_anchor");
            println!("  get_anchor result: {:?}", anchor);

            // Verify hit count tracking
            let hit_count = new_ctx.get_hit_count(node_name);
            println!("  get_hit_count({}) = {:?}", node_name, hit_count);

            let clear_result = new_ctx.clear_hit_count(node_name);
            println!("  clear_hit_count result: {:?}", clear_result);

            // Verify runtime image overriding in context
            let mut test_image_buf = maa_framework::buffer::MaaImageBuffer::new()
                .expect("Failed to create image buffer");
            let raw_data = vec![0u8; 100 * 100 * 3];
            let _ = test_image_buf.set_raw_data(&raw_data, 100, 100, 16); // 16 = CV_8UC3
            let override_img = new_ctx.override_image("test_image", &test_image_buf);
            println!("  override_image result: {:?}", override_img);

            // Verify access to the underlying tasker handle from the context
            let tasker_handle = new_ctx.tasker_handle();
            println!("  context.tasker_handle: {:?}", tasker_handle);
            if !tasker_handle.is_null() {
                println!("  PASS: context.tasker_handle() is valid");
            }
            // Verify independent task job status query from context
            let task_job = new_ctx.get_task_job();
            match task_job.get(false) {
                Ok(Some(task_detail)) => {
                    println!("  get_task_job entry: {}", task_detail.entry);
                }
                Ok(None) => {
                    println!("  get_task_job returned None");
                }
                Err(e) => {
                    println!("  get_task_job failed: {}", e);
                }
            }
        }

        // Verify task ID retrieval
        let task_id = context.task_id();
        println!("  context.task_id: {}", task_id);

        ANALYZED.store(true, Ordering::SeqCst);

        Some((
            sys::MaaRect {
                x: 11,
                y: 4,
                width: 5,
                height: 14,
            },
            r#"{"message": "Hello World!"}"#.to_string(),
        ))
    }
}

struct MyAction;

impl CustomAction for MyAction {
    fn run(
        &self,
        context: &Context,
        _task_id: sys::MaaTaskId,
        node_name: &str,
        custom_action_name: &str,
        _custom_action_param: &str,
        _reco_id: sys::MaaRecoId,
        _box_rect: &sys::MaaRect,
    ) -> bool {
        println!(
            "on MyAction.run, context: {:?}, node: {}, action: {}",
            context.raw(),
            node_name,
            custom_action_name
        );

        RUNNED.store(true, Ordering::SeqCst);

        true
    }
}

// ============================================================================
// Custom Controller for testing
// ============================================================================

struct MyController {
    count: Arc<AtomicUsize>,
    image: Vec<u8>,
}

impl MyController {
    fn new(count: Arc<AtomicUsize>) -> Self {
        let test_res_dir = common::get_test_resources_dir();
        let screenshot_dir = test_res_dir.join("Screenshot");

        let mut image = Vec::new();
        if let Ok(entries) = std::fs::read_dir(screenshot_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "png") {
                    if let Ok(data) = std::fs::read(path) {
                        image = data;
                        break;
                    }
                }
            }
        }

        if image.is_empty() {
            println!("WARNING: No PNG found in Screenshot dir, using empty vec (will fail task)");
        }

        Self { count, image }
    }
}

impl CustomControllerCallback for MyController {
    fn connect(&self) -> bool {
        println!("  on MyController.connect");
        self.count.fetch_add(1, Ordering::SeqCst);
        true
    }

    fn request_uuid(&self) -> Option<String> {
        println!("  on MyController.request_uuid");
        Some("12345678".to_string())
    }

    fn start_app(&self, intent: &str) -> bool {
        println!("  on MyController.start_app: {}", intent);
        self.count.fetch_add(1, Ordering::SeqCst);
        true
    }

    fn stop_app(&self, intent: &str) -> bool {
        println!("  on MyController.stop_app: {}", intent);
        self.count.fetch_add(1, Ordering::SeqCst);
        true
    }

    fn screencap(&self) -> Option<Vec<u8>> {
        println!("  on MyController.screencap");
        self.count.fetch_add(1, Ordering::SeqCst);
        if self.image.is_empty() {
            Some(Vec::new())
        } else {
            Some(self.image.clone())
        }
    }

    fn click(&self, x: i32, y: i32) -> bool {
        println!("  on MyController.click: {}, {}", x, y);
        self.count.fetch_add(1, Ordering::SeqCst);
        true
    }

    fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration: i32) -> bool {
        println!(
            "  on MyController.swipe: {}, {} -> {}, {}, {}",
            x1, y1, x2, y2, duration
        );
        self.count.fetch_add(1, Ordering::SeqCst);
        true
    }

    fn touch_down(&self, contact: i32, x: i32, y: i32, pressure: i32) -> bool {
        println!(
            "  on MyController.touch_down: {}, {}, {}, {}",
            contact, x, y, pressure
        );
        self.count.fetch_add(1, Ordering::SeqCst);
        true
    }

    fn touch_move(&self, contact: i32, x: i32, y: i32, pressure: i32) -> bool {
        println!(
            "  on MyController.touch_move: {}, {}, {}, {}",
            contact, x, y, pressure
        );
        self.count.fetch_add(1, Ordering::SeqCst);
        true
    }

    fn touch_up(&self, contact: i32) -> bool {
        println!("  on MyController.touch_up: {}", contact);
        self.count.fetch_add(1, Ordering::SeqCst);
        true
    }

    fn click_key(&self, keycode: i32) -> bool {
        println!("  on MyController.click_key: {}", keycode);
        self.count.fetch_add(1, Ordering::SeqCst);
        true
    }

    fn input_text(&self, text: &str) -> bool {
        println!("  on MyController.input_text: {}", text);
        self.count.fetch_add(1, Ordering::SeqCst);
        true
    }

    fn key_down(&self, keycode: i32) -> bool {
        println!("  on MyController.key_down: {}", keycode);
        self.count.fetch_add(1, Ordering::SeqCst);
        true
    }

    fn key_up(&self, keycode: i32) -> bool {
        println!("  on MyController.key_up: {}", keycode);
        self.count.fetch_add(1, Ordering::SeqCst);
        true
    }

    fn scroll(&self, dx: i32, dy: i32) -> bool {
        println!("  on MyController.scroll: {}, {}", dx, dy);
        self.count.fetch_add(1, Ordering::SeqCst);
        true
    }
}

// ============================================================================
// Resource API Tests
// ============================================================================

#[test]
fn test_resource_inference_settings() {
    println!("\n=== test_resource_inference_settings ===");

    let r1 = Resource::new().expect("Failed to create resource");
    // Test inference device settings (should not panic)
    let _ = r1.use_cpu();
    let _ = r1.use_directml(0);
    let _ = r1.use_auto_ep();

    println!("  PASS: inference settings");
}

#[test]
fn test_resource_loading() {
    println!("\n=== test_resource_loading ===");

    let _ = init_test_env();

    let resource = Resource::new().expect("Failed to create resource");

    // Test invalid path - wait for task completion to verify no crash
    if let Ok(job) = resource.post_bundle("C:/_maafw_testing_/aaabbbccc") {
        let status = job.wait();
        println!("  invalid path status: {:?} (expected failure)", status);
    }
    println!(
        "  resource.loaded (after invalid path): {}",
        resource.loaded()
    );

    // Verify loading valid resources (requires git submodule to be initialized)
    let test_res_dir = get_test_resources_dir();
    let resource_path = test_res_dir.join("resource");

    assert!(
        resource_path.exists(),
        "Test resources MUST exist at {:?}. Run: git submodule update --init",
        resource_path
    );

    let job = resource
        .post_bundle(resource_path.to_str().unwrap())
        .expect("post_bundle MUST succeed");
    let status = job.wait();
    assert!(status.done(), "Resource loading MUST complete");
    assert!(resource.loaded(), "Resource MUST be loaded");
    println!(
        "  resource.loaded: {}, status: {:?}",
        resource.loaded(),
        status
    );

    println!("  PASS: resource loading (STRICT)");
}

#[test]
fn test_resource_custom_registration() {
    println!("\n=== test_resource_custom_registration ===");

    let resource = Resource::new().expect("Failed to create resource");

    // Register custom recognition and action
    resource
        .register_custom_recognition("MyRec", Box::new(MyRecognition))
        .expect("Failed to register recognition");
    resource
        .register_custom_action("MyAct", Box::new(MyAction))
        .expect("Failed to register action");

    // Test custom lists
    let reco_list = resource
        .custom_recognition_list()
        .expect("Failed to get reco list");
    let action_list = resource
        .custom_action_list()
        .expect("Failed to get action list");
    println!("  custom_recognition_list: {:?}", reco_list);
    println!("  custom_action_list: {:?}", action_list);

    assert!(
        reco_list.contains(&"MyRec".to_string()),
        "MyRec should be registered"
    );
    assert!(
        action_list.contains(&"MyAct".to_string()),
        "MyAct should be registered"
    );

    // Test unregister
    resource
        .unregister_custom_recognition("MyRec")
        .expect("Failed to unregister recognition");
    resource
        .unregister_custom_action("MyAct")
        .expect("Failed to unregister action");

    let reco_list_after = resource
        .custom_recognition_list()
        .expect("Failed to get reco list");
    let action_list_after = resource
        .custom_action_list()
        .expect("Failed to get action list");
    assert!(
        !reco_list_after.contains(&"MyRec".to_string()),
        "MyRec should be unregistered"
    );
    assert!(
        !action_list_after.contains(&"MyAct".to_string()),
        "MyAct should be unregistered"
    );

    // Test clear
    resource
        .register_custom_recognition("MyRec2", Box::new(MyRecognition))
        .expect("Failed to register recognition");
    resource
        .clear_custom_recognition()
        .expect("Failed to clear");
    let reco_list_cleared = resource
        .custom_recognition_list()
        .expect("Failed to get reco list");
    assert!(reco_list_cleared.is_empty(), "Should be empty after clear");

    println!("  PASS: custom registration");
}

#[test]
fn test_resource_sink_operations() {
    println!("\n=== test_resource_sink_operations ===");

    let resource = Resource::new().expect("Failed to create resource");

    // Add sink
    let sink_id = resource
        .add_sink(|msg, details| {
            println!("  [ResourceSink] msg: {}, details: {}", msg, details);
        })
        .expect("Failed to add sink");
    println!("  sink_id: {:?}", sink_id);

    // Remove sink
    resource.remove_sink(sink_id);

    // Clear sinks
    let _ = resource.add_sink(|_, _| {});
    resource.clear_sinks();

    println!("  PASS: sink operations");
}

#[test]
fn test_resource_event_sink_structured() {
    println!("\n=== test_resource_event_sink_structured ===");

    use maa_framework::event_sink::EventSink;
    use maa_framework::notification::MaaEvent;
    use std::sync::{Arc, Mutex};

    struct TestSink {
        events: Arc<Mutex<Vec<String>>>,
    }

    impl EventSink for TestSink {
        fn on_event(&self, _handle: maa_framework::common::MaaId, event: &MaaEvent) {
            let mut events = self.events.lock().unwrap();
            match event {
                MaaEvent::ResourceLoadingStarting(detail) => {
                    println!("  [StructuredSink] Loading Starting: {}", detail.path);
                    events.push("Starting".to_string());
                }
                MaaEvent::ResourceLoadingSucceeded(detail) => {
                    println!("  [StructuredSink] Loading Succeeded: {}", detail.path);
                    events.push("Succeeded".to_string());
                }
                MaaEvent::ResourceLoadingFailed(detail) => {
                    println!("  [StructuredSink] Loading Failed: {}", detail.path);
                    events.push("Failed".to_string());
                }
                _ => {}
            }
        }
    }

    let resource = Resource::new().expect("Failed to create resource");
    let received_events = Arc::new(Mutex::new(Vec::new()));

    // Register typed sink
    resource
        .add_event_sink(Box::new(TestSink {
            events: received_events.clone(),
        }))
        .expect("Failed to add event sink");

    // Load resources to trigger events
    let test_res_dir = get_test_resources_dir();
    let resource_path = test_res_dir.join("resource");

    resource
        .post_bundle(resource_path.to_str().unwrap())
        .expect("post_bundle")
        .wait();

    // Verify events received
    let events = received_events.lock().unwrap();
    println!("  Received events: {:?}", *events);
    assert!(!events.is_empty(), "Should receive resource events");
    assert!(events.contains(&"Starting".to_string()));
    assert!(events.contains(&"Succeeded".to_string()));

    println!("  PASS: resource event sink structured");
}

#[test]
fn test_resource_node_operations() {
    println!("\n=== test_resource_node_operations ===");

    let _ = init_test_env();

    let resource = Resource::new().expect("Failed to create resource");

    // Load test resources (requires git submodule to be initialized)
    let test_res_dir = get_test_resources_dir();
    let resource_path = test_res_dir.join("resource");

    assert!(
        resource_path.exists(),
        "Test resources MUST exist at {:?}. Run: git submodule update --init",
        resource_path
    );

    resource
        .post_bundle(resource_path.to_str().unwrap())
        .expect("post_bundle MUST succeed")
        .wait();

    assert!(resource.loaded(), "Resource MUST be loaded");

    // Verify retrieval of the full node list
    let node_list = resource.node_list().expect("node_list MUST work");
    println!("  node_list count: {}", node_list.len());
    assert!(!node_list.is_empty(), "node_list MUST NOT be empty");

    // Verify retrieval of node property data
    let first_node = node_list
        .first()
        .expect("Node list MUST have at least one node");
    let node_data = resource
        .get_node_data(first_node)
        .expect("get_node_data MUST NOT error")
        .expect("get_node_data MUST return Some");
    assert!(
        node_data.contains("recognition"),
        "node_data MUST contain 'recognition'"
    );
    println!(
        "  get_node_data({}) verified: {} bytes",
        first_node,
        node_data.len()
    );

    // Verify resource hash availability
    let hash = resource.hash().expect("hash MUST work");
    assert!(!hash.is_empty(), "Hash MUST NOT be empty");
    println!("  resource.hash: {}", hash);

    // Verify dynamic pipeline override functionality
    let pipeline_override = r#"{"TestOverride": {"action": "DoNothing"}}"#;
    resource
        .override_pipeline(pipeline_override)
        .expect("override_pipeline MUST succeed");
    println!("  override_pipeline: OK");

    // Verify dynamic next-node list override
    resource
        .override_next("TestOverride", &["SomeNode"])
        .expect("override_next MUST succeed");
    println!("  override_next: OK");

    // Verify dynamic resource image override
    let mut image_buf =
        maa_framework::buffer::MaaImageBuffer::new().expect("Failed to create image buffer");
    // Create a simple 10x10 black image
    let img_data = vec![0u8; 10 * 10 * 3];
    image_buf
        .set_raw_data(&img_data, 10, 10, 16) // 16 = CV_8UC3
        .expect("set_raw_data MUST succeed");

    resource
        .override_image("TestImage", &image_buf)
        .expect("override_image MUST succeed");
    println!("  override_image: OK");

    // Verify getting default parameters
    let ocr_default = resource
        .get_default_recognition_param("OCR")
        .expect("get_default_recognition_param failed");
    println!("  ocr_default: {:?}", ocr_default);
    assert!(ocr_default.is_some(), "OCR default parameters should exist");

    let click_default = resource
        .get_default_action_param("Click")
        .expect("get_default_action_param failed");
    println!("  click_default: {:?}", click_default);
    assert!(
        click_default.is_some(),
        "Click default parameters should exist"
    );

    println!("  PASS: node operations (STRICT)");
}

// ============================================================================
// Controller API Tests
// ============================================================================

#[test]
fn test_dbg_controller_creation() {
    println!("\n=== test_dbg_controller_creation ===");

    let test_res_dir = get_test_resources_dir();
    let screenshot_dir = test_res_dir.join("Screenshot");

    if !screenshot_dir.exists() {
        panic!(
            "Screenshot directory not found at {:?}. Run: git submodule update --init",
            screenshot_dir
        );
    }

    // Create a user dir in temp
    let user_dir = std::env::temp_dir().join("maa_test_user");
    std::fs::create_dir_all(&user_dir).ok();

    let controller = Controller::new_dbg(
        screenshot_dir.to_str().unwrap(),
        user_dir.to_str().unwrap(),
        sys::MaaDbgControllerType_CarouselImage as u64,
        "{}",
    );

    match controller {
        Ok(_) => println!("  PASS: DbgController creation"),
        Err(e) => {
            // MaaDbgControlUnit.dll is not included in release packages
            panic!(
                "DbgController not available ({}). Ensure MaaDbgControlUnit.dll is present.",
                e
            );
        }
    }
}

#[test]
fn test_controller_connection() {
    println!("\n=== test_controller_connection ===");

    let _ = init_test_env();

    let test_res_dir = get_test_resources_dir();
    let screenshot_dir = test_res_dir.join("Screenshot");

    if !screenshot_dir.exists() {
        panic!(
            "Screenshot directory not found at {:?}. Run: git submodule update --init",
            screenshot_dir
        );
    }

    let user_dir = std::env::temp_dir().join("maa_test_user");
    std::fs::create_dir_all(&user_dir).ok();

    let controller = match Controller::new_dbg(
        screenshot_dir.to_str().unwrap(),
        user_dir.to_str().unwrap(),
        sys::MaaDbgControllerType_CarouselImage as u64,
        "{}",
    ) {
        Ok(c) => c,
        Err(e) => {
            panic!(
                "DbgController not available ({}). Ensure MaaDbgControlUnit.dll is present.",
                e
            );
        }
    };

    // Test connection
    let conn_id = controller
        .post_connection()
        .expect("Failed to post connection");
    let status = controller.wait(conn_id);
    println!("  connection status: {:?}", status);
    println!("  connected: {}", controller.connected());

    // Test UUID
    let uuid = controller.uuid();
    println!("  uuid: {:?}", uuid);

    println!("  PASS: controller connection");
}

#[test]
fn test_controller_screencap() {
    println!("\n=== test_controller_screencap ===");

    let _ = init_test_env();

    let test_res_dir = get_test_resources_dir();
    let screenshot_dir = test_res_dir.join("Screenshot");

    if !screenshot_dir.exists() {
        panic!(
            "Screenshot directory not found at {:?}. Run: git submodule update --init",
            screenshot_dir
        );
    }

    let user_dir = std::env::temp_dir().join("maa_test_user");
    std::fs::create_dir_all(&user_dir).ok();

    let controller = match Controller::new_dbg(
        screenshot_dir.to_str().unwrap(),
        user_dir.to_str().unwrap(),
        sys::MaaDbgControllerType_CarouselImage as u64,
        "{}",
    ) {
        Ok(c) => c,
        Err(e) => {
            panic!(
                "DbgController not available ({}). Ensure MaaDbgControlUnit.dll is present.",
                e
            );
        }
    };

    // Connect first
    let conn_id = controller
        .post_connection()
        .expect("Failed to post connection");
    controller.wait(conn_id);

    // Verify setting screenshot parameters
    controller
        .set_screenshot_target_long_side(1920)
        .expect("set_screenshot_target_long_side failed");
    controller
        .set_screenshot_target_short_side(1080)
        .expect("set_screenshot_target_short_side failed");
    controller
        .set_screenshot_use_raw_size(false)
        .expect("set_screenshot_use_raw_size failed");
    println!("  screenshot options set: OK");

    // Test screencap
    let cap_id = controller
        .post_screencap()
        .expect("Failed to post screencap");
    let status = controller.wait(cap_id);
    println!("  screencap status: {:?}", status);

    // Test resolution
    let resolution = controller.resolution();
    println!("  resolution: {:?}", resolution);

    println!("  PASS: controller screencap");
}

#[test]
fn test_controller_sink_operations() {
    println!("\n=== test_controller_sink_operations ===");

    let test_res_dir = get_test_resources_dir();
    let screenshot_dir = test_res_dir.join("Screenshot");

    if !screenshot_dir.exists() {
        panic!(
            "Screenshot directory not found at {:?}. Run: git submodule update --init",
            screenshot_dir
        );
    }

    let user_dir = std::env::temp_dir().join("maa_test_user");
    std::fs::create_dir_all(&user_dir).ok();

    let controller = match Controller::new_dbg(
        screenshot_dir.to_str().unwrap(),
        user_dir.to_str().unwrap(),
        sys::MaaDbgControllerType_CarouselImage as u64,
        "{}",
    ) {
        Ok(c) => c,
        Err(e) => {
            panic!(
                "DbgController not available ({}). Ensure MaaDbgControlUnit.dll is present.",
                e
            );
        }
    };

    // Add sink
    let sink_id = controller
        .add_sink(|msg, details| {
            println!("  [ControllerSink] msg: {}, details: {}", msg, details);
        })
        .expect("Failed to add sink");
    println!("  sink_id: {:?}", sink_id);

    // Remove and clear
    controller.remove_sink(sink_id);
    let _ = controller.add_sink(|_, _| {});
    controller.clear_sinks();

    println!("  PASS: controller sink operations");
}

// ============================================================================
// Tasker API Tests
// ============================================================================

#[test]
fn test_tasker_global_options() {
    println!("\n=== test_tasker_global_options ===");

    // Test global options (static methods equivalent)
    maa_framework::set_save_draw(true).expect("Failed to set save_draw");
    maa_framework::set_stdout_level(sys::MaaLoggingLevelEnum_MaaLoggingLevel_All as i32)
        .expect("Failed to set stdout level");
    maa_framework::configure_logging(".").expect("Failed to configure logging");
    maa_framework::set_debug_mode(true).expect("Failed to set debug mode");
    maa_framework::set_save_on_error(true).expect("Failed to set save on error");
    maa_framework::set_draw_quality(85).expect("Failed to set draw quality");
    maa_framework::set_reco_image_cache_limit(4096).expect("Failed to set reco cache limit");

    println!("  PASS: global options");
}

#[test]
fn test_tasker_api() {
    println!("\n=== test_tasker_api ===");

    let _ = init_test_env();

    // Create tasker
    let tasker = Tasker::new().expect("Failed to create tasker");
    println!("  tasker created");

    // Create resource
    let resource = Resource::new().expect("Failed to create resource");

    // Load test resources
    let test_res_dir = get_test_resources_dir();
    let resource_path = test_res_dir.join("resource");
    assert!(
        resource_path.exists(),
        "Test resources MUST exist at {:?}. Run: git submodule update --init",
        resource_path
    );

    let job = resource
        .post_bundle(resource_path.to_str().unwrap())
        .expect("post_bundle MUST succeed");
    let status = job.wait();
    assert!(status.succeeded(), "Resource loading failed");

    // Register custom components
    resource
        .register_custom_recognition("MyRec", Box::new(MyRecognition))
        .expect("Failed to register MyRec");
    resource
        .register_custom_action("MyAct", Box::new(MyAction))
        .expect("Failed to register MyAct");

    // Create controller (Use MyController for reliable testing without external DLLs)
    let count = Arc::new(AtomicUsize::new(0));
    let my_ctrl = MyController::new(count.clone());
    let controller = Controller::new_custom(my_ctrl).expect("Failed to create custom controller");

    // Connect
    let conn_id = controller.post_connection().unwrap();
    controller.wait(conn_id);

    // Bind
    tasker
        .bind_resource(&resource)
        .expect("Failed to bind resource");
    tasker
        .bind_controller(&controller)
        .expect("Failed to bind controller");

    println!("  inited: {}", tasker.inited());

    // Execute Task to trigger callbacks
    let ppover = r#"{
        "Entry": {"next": "Rec"},
        "Rec": {
            "recognition": "Custom",
            "custom_recognition": "MyRec",
            "action": "Custom",
            "custom_action": "MyAct",
            "custom_action_param": "Test111222333"
        }
    }"#;
    // Post task
    let task_job = tasker
        .post_task("Entry", ppover)
        .expect("Failed to post task");

    // Verify task execution status and detailed results
    let status = task_job.wait();
    assert!(status.succeeded(), "Task execution failed");
    println!("  task status: {:?}", status);

    // Validate the complete task detail structure, including nodes, recognitions, and actions
    let detail = task_job
        .get(false)
        .expect("Should get details")
        .expect("Detail should be Some");
    println!("  task detail entry: {}", detail.entry);
    println!("  task detail status: {:?}", detail.status);
    println!("  nodes count: {}", detail.nodes.len());

    assert!(!detail.nodes.is_empty(), "Task should execute nodes");

    for node_opt in &detail.nodes {
        let node_detail = node_opt.as_ref().expect("Node detail should exist");

        println!(
            "    Node: {} (id: not exposed), completed: {}, reco_id: {}, act_id: {}",
            node_detail.node_name, node_detail.completed, node_detail.reco_id, node_detail.act_id
        );

        // Verify Recognition Detail
        if let Some(reco_detail) = &node_detail.recognition {
            println!(
                "      Reco: {} (algo: {:?}, hit: {})",
                reco_detail.node_name, reco_detail.algorithm, reco_detail.hit
            );
        }

        // Verify Action Detail
        if let Some(act_detail) = &node_detail.action {
            println!(
                "      Action: {} (method: {:?}, success: {})",
                act_detail.node_name, act_detail.action, act_detail.success
            );
        }
    }

    assert!(
        ANALYZED.load(Ordering::SeqCst),
        "MyRecognition.analyze should be called"
    );
    assert!(
        RUNNED.load(Ordering::SeqCst),
        "MyAction.run should be called"
    );

    // Test clear_cache
    tasker.clear_cache().expect("Failed to clear cache");

    // Test override_pipeline (via job object)
    let override_result = task_job
        .override_pipeline(r#"{"Entry": {"next": []}}"#)
        .expect("Failed to override pipeline");
    println!("  task_job.override_pipeline result: {}", override_result);

    println!("  PASS: tasker api (STRICT)");
}

#[test]
fn test_tasker_sink_operations() {
    println!("\n=== test_tasker_sink_operations ===");

    use maa_framework::event_sink::EventSink;
    use maa_framework::notification::MaaEvent;

    struct TestSink;
    impl EventSink for TestSink {
        fn on_event(&self, _handle: maa_framework::common::MaaId, event: &MaaEvent) {
            println!("  [ContextEventSink] event: {:?}", event);
        }
    }

    let tasker = Tasker::new().expect("Failed to create tasker");

    // Add tasker sink
    let tasker_sink_id = tasker
        .add_sink(|msg, details| {
            println!("  [TaskerSink] msg: {}, details: {}", msg, details);
        })
        .expect("Failed to add tasker sink");

    // Add context sink
    let context_sink_id = tasker
        .add_context_sink(|msg, details| {
            println!("  [ContextSink] msg: {}, details: {}", msg, details);
        })
        .expect("Failed to add context sink");

    // Add typed context event sink
    let typed_sink_id = tasker
        .add_context_event_sink(Box::new(TestSink))
        .expect("Failed to add context event sink");

    println!(
        "  tasker_sink_id: {:?}, context_sink_id: {:?}, typed_sink_id: {:?}",
        tasker_sink_id, context_sink_id, typed_sink_id
    );

    // Remove sinks
    tasker.remove_sink(tasker_sink_id);
    tasker.remove_context_sink(context_sink_id);
    tasker.remove_context_sink(typed_sink_id);

    // Clear sinks
    let _ = tasker.add_sink(|_, _| {});
    let _ = tasker.add_context_sink(|_, _| {});
    tasker.clear_sinks();
    tasker.clear_context_sinks();

    println!("  PASS: tasker sink operations");
}

#[test]
fn test_tasker_state_queries() {
    println!("\n=== test_tasker_state_queries ===");

    let tasker = Tasker::new().expect("Failed to create tasker");

    println!("  running: {}", tasker.running());
    println!("  stopping: {}", tasker.stopping());

    // Test clear cache
    tasker.clear_cache().expect("Failed to clear cache");

    println!("  PASS: tasker state queries");
}

// ============================================================================
// CustomController Tests
// ============================================================================

#[test]
fn test_custom_controller_operations() {
    println!("\n=== test_custom_controller_operations ===");

    let count = Arc::new(AtomicUsize::new(0));
    let my_ctrl = MyController::new(count.clone());

    let controller = Controller::new_custom(my_ctrl).expect("Failed to create custom controller");

    // Test connection
    let conn_id = controller
        .post_connection()
        .expect("Failed to post connection");
    let status = controller.wait(conn_id);
    println!("  connection status: {:?}", status);

    // Test UUID
    let uuid = controller.uuid();
    println!("  uuid: {:?}", uuid);

    // Test various operations - wait for each to complete
    if let Ok(id) = controller.post_click(100, 200) {
        controller.wait(id);
    }
    if let Ok(id) = controller.post_swipe(100, 200, 300, 400, 200) {
        controller.wait(id);
    }
    if let Ok(id) = controller.post_touch_down(1, 100, 100, 0) {
        controller.wait(id);
    }
    if let Ok(id) = controller.post_touch_move(1, 200, 200, 0) {
        controller.wait(id);
    }
    if let Ok(id) = controller.post_touch_up(1) {
        controller.wait(id);
    }
    if let Ok(id) = controller.post_click_key(32) {
        controller.wait(id);
    }
    if let Ok(id) = controller.post_input_text("Hello World!") {
        controller.wait(id);
    }

    // Verify callbacks are executed
    let executed_count = count.load(Ordering::SeqCst);
    println!("  total callbacks executed: {}", executed_count);
    assert!(
        executed_count > 0,
        "Custom controller callbacks executed: {}",
        executed_count
    );

    println!("  PASS: custom controller operations");
}

// ============================================================================
// Toolkit Tests
// ============================================================================

#[test]
fn test_toolkit_find_devices() {
    println!("\n=== test_toolkit_find_devices ===");

    let _ = init_test_env();

    // Find ADB devices - may return empty or error, both are acceptable
    match Toolkit::find_adb_devices() {
        Ok(devices) => println!("  adb devices: {}", devices.len()),
        Err(e) => println!("  adb devices: error ({:?})", e),
    }

    // Find desktop windows - may not be implemented on all platforms
    // CI environments often don't have a desktop, so we just verify no crash
    match Toolkit::find_desktop_windows() {
        Ok(list) => {
            println!("  desktop windows: {}", list.len());
            for win in list.iter().take(3) {
                let name = if win.window_name.len() > 30 {
                    &win.window_name[..30]
                } else {
                    &win.window_name
                };
                println!("    - {}", name);
            }
        }
        Err(e) => {
            // Not implemented or no access to desktop - acceptable in CI
            println!("  desktop windows: unavailable ({:?})", e);
        }
    }

    println!("  PASS: toolkit API calls completed without crash");
}

// ============================================================================
// Version Test
// ============================================================================

#[test]
fn test_maa_version() {
    println!("\n=== test_maa_version ===");

    let version = maa_framework::maa_version();
    println!("  MaaFw Version: {}", version);
    assert!(!version.is_empty(), "Version should not be empty");

    println!("  PASS: version");
}

// ============================================================================
// Native Controller Integration Test
// ============================================================================

/// Test Tasker with native DbgController to verify C++ integration.
///
/// This complements test_tasker_api (which uses CustomController) by testing
/// the path where Rust drives a C++ native controller component.
#[test]
fn test_tasker_with_native_dbg_controller() {
    println!("\n=== test_tasker_with_native_dbg_controller ===");

    let _ = init_test_env();

    let test_res_dir = get_test_resources_dir();
    let screenshot_dir = test_res_dir.join("Screenshot");
    let resource_path = test_res_dir.join("resource");
    let user_dir = std::env::temp_dir().join("maa_test_user_native");
    std::fs::create_dir_all(&user_dir).ok();

    assert!(
        screenshot_dir.exists(),
        "Screenshot dir MUST exist at {:?}",
        screenshot_dir
    );

    // Create native DbgController
    let controller = match Controller::new_dbg(
        screenshot_dir.to_str().unwrap(),
        user_dir.to_str().unwrap(),
        sys::MaaDbgControllerType_CarouselImage as u64,
        "{}",
    ) {
        Ok(c) => c,
        Err(e) => {
            println!("  SKIPPED: MaaDbgControlUnit not available ({})", e);
            return;
        }
    };

    // Connect
    let conn_id = controller.post_connection().unwrap();
    let conn_status = controller.wait(conn_id);
    assert!(conn_status.succeeded(), "DbgController connection failed");
    println!("  DbgController connected");

    // Prepare Resource
    let resource = Resource::new().unwrap();
    let res_job = resource
        .post_bundle(resource_path.to_str().unwrap())
        .expect("post_bundle MUST succeed");
    let res_status = res_job.wait();
    assert!(res_status.succeeded(), "Resource loading failed");

    // Create and bind Tasker
    let tasker = Tasker::new().unwrap();
    tasker.bind_resource(&resource).unwrap();
    tasker.bind_controller(&controller).unwrap();

    assert!(tasker.inited(), "Tasker must be initialized");
    println!("  Tasker initialized with native DbgController");

    // Run a simple task - just verify the path works
    let ppover = r#"{"TestEntry": {"action": "Click", "target": [10, 10, 20, 20]}}"#;
    let job = tasker.post_task("TestEntry", ppover).unwrap();
    let status = job.wait();

    println!("  Task status: {:?}", status);
    assert!(
        status.succeeded(),
        "Task with native DbgController should succeed"
    );

    println!("  PASS: tasker with native DbgController");
}

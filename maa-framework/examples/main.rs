//! Comprehensive MaaFramework Rust SDK Demo
//!
//! This example demonstrates all major features of the Rust binding,
//! matching the functionality of the official Python demo.
//!
//! # Features Covered
//! - Toolkit initialization and device discovery
//! - ADB and Win32 controller creation
//! - Resource loading and custom component registration
//! - Tasker binding and task execution
//! - Custom Recognition and Custom Action implementation
//! - Context API usage within custom components
//! - Event sink callbacks

use maa_framework::buffer::MaaImageBuffer;
use maa_framework::context::Context;
use maa_framework::controller::Controller;
use maa_framework::custom::{CustomAction, CustomRecognition};
use maa_framework::resource::Resource;
use maa_framework::tasker::Tasker;
use maa_framework::toolkit::Toolkit;
use maa_framework::{common, sys};
use std::path::Path;

// ============================================================================
// Custom Recognition Implementation
// ============================================================================

/// Example custom recognition demonstrating Context API usage.
///
/// This shows how to:
/// - Run sub-pipeline recognition
/// - Clone context for independent operations
/// - Override pipeline configurations
/// - Check tasker stopping state
struct MyRecognition;

impl CustomRecognition for MyRecognition {
    fn analyze(
        &self,
        context: &Context,
        _task_id: sys::MaaTaskId,
        node_name: &str,
        custom_recognition_name: &str,
        custom_recognition_param: &str,
        _image: &maa_framework::buffer::MaaImageBuffer,
        _roi: &maa_framework::common::Rect,
    ) -> Option<(maa_framework::common::Rect, String)> {
        println!(
            "[MyRecognition] analyze called: node={}, name={}, param={}",
            node_name, custom_recognition_name, custom_recognition_param
        );

        // --- Context API Demo ---

        // 1. Run sub-pipeline recognition with override (requires OCR model in resource)
        if Path::new("sample/resource").exists() {
            let pp_override =
                r#"{"MyCustomOCR": {"recognition": "OCR", "roi": [100, 100, 200, 300]}}"#;
            let reco_id = context.run_recognition("MyCustomOCR", pp_override, _image);
            println!("  run_recognition result: {:?}", reco_id);
        } else {
            println!("  找不到 sample/resource，跳过 run_recognition 演示...");
        }

        // 2. Take a new screenshot via tasker's controller
        let tasker_ptr = context.tasker_handle();
        if !tasker_ptr.is_null() {
            // Note: In real usage, you would use the Tasker's controller reference
            println!("  tasker handle available for controller access");
        }

        // 3. Async click - post now, will wait later
        // (In real code, you'd get controller from context.tasker)
        println!("  [Demo] Would post async click at (10, 20)");

        // 4. Override pipeline for all subsequent operations in this context
        let _ = context.override_pipeline(r#"{"MyCustomOCR": {"roi": [1, 1, 114, 514]}}"#);

        // 5. Check if tasker is stopping - IMPORTANT for responsive cancellation
        // Note: Context provides tasker_handle() which returns a raw pointer.
        // In production code, you should wrap this or use the Tasker instance directly.
        // Here we demonstrate the raw API call for completeness.
        let tasker_ptr = context.tasker_handle();
        if !tasker_ptr.is_null() {
            // Using sys:: directly requires unsafe - this matches C API usage
            let is_stopping = unsafe { sys::MaaTaskerStopping(tasker_ptr) != 0 };
            if is_stopping {
                println!("  Task is stopping, returning early!");
                return Some((
                    common::Rect {
                        x: 0,
                        y: 0,
                        width: 0,
                        height: 0,
                    },
                    r#"{"status": "Task Stopped"}"#.to_string(),
                ));
            }
        }

        // 6. Wait for the async click to complete
        println!("  [Demo] Async click would complete here");

        // 7. Clone context for independent operations (modifications won't affect original)
        if Path::new("sample/resource").exists() {
            if let Ok(new_ctx) = context.clone_context() {
                let _ = new_ctx.override_pipeline(r#"{"MyCustomOCR": {"roi": [100, 200, 300, 400]}}"#);

                if let Ok(img_buf) = MaaImageBuffer::new() {
                    let reco_id = new_ctx.run_recognition("MyCustomOCR", "{}", &img_buf);
                    if reco_id.is_ok() {
                        println!("  [Demo] clone_context + run_recognition 完成");
                    }
                }
            }
        } else {
            println!("  找不到 sample/resource，跳过 clone_context 演示...");
        }

        // 8. Get current task ID
        let task_id = context.task_id();
        println!("  current task_id: {}", task_id);

        // 9. Get task job for result retrieval
        let task_job = context.get_task_job();
        if let Ok(Some(detail)) = task_job.get(false) {
            println!("  task entry: {}", detail.entry);
        }

        // Return recognition result: bounding box + detail JSON
        Some((
            common::Rect {
                x: 0,
                y: 0,
                width: 100,
                height: 100,
            },
            r#"{"message": "Hello World!"}"#.to_string(),
        ))
    }
}

// ============================================================================
// Custom Action Implementation
// ============================================================================

/// Example custom action.
///
/// Actions are executed after a successful recognition.
struct MyAction;

impl CustomAction for MyAction {
    fn run(
        &self,
        context: &Context,
        _task_id: sys::MaaTaskId,
        node_name: &str,
        custom_action_name: &str,
        custom_action_param: &str,
        reco_id: sys::MaaRecoId,
        box_rect: &maa_framework::common::Rect,
    ) -> bool {
        println!(
            "[MyAction] run called: node={}, name={}, param={}",
            node_name, custom_action_name, custom_action_param
        );
        println!(
            "  reco_id={}, box=({}, {}, {}, {})",
            reco_id, box_rect.x, box_rect.y, box_rect.width, box_rect.height
        );

        if Path::new("sample/resource").exists() {
            let _ = context.run_action(
                "Click",
                "{}",
                &common::Rect {
                    x: 114,
                    y: 514,
                    width: 100,
                    height: 100,
                },
                "{}",
            );
        }

        true
    }
}

// ============================================================================
// Main Demo
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MaaFramework Rust SDK Demo ===\n");

    #[cfg(feature = "dynamic")]
    maa_framework::ensure_library_loaded()?;

    // -------------------------------------------------------------------------
    // 1. Initialize Toolkit
    // -------------------------------------------------------------------------
    let user_path = "./";
    Toolkit::init_option(user_path, "{}")?;
    println!("[1] Toolkit initialized with user_path: {}", user_path);

    // -------------------------------------------------------------------------
    // 2. Device Discovery
    // -------------------------------------------------------------------------

    // ADB devices
    println!("\n[2] Scanning for ADB devices...");
    let adb_devices = Toolkit::find_adb_devices()?;
    if adb_devices.is_empty() {
        println!("    No ADB device found.");
    } else {
        for device in &adb_devices {
            println!("    Found: {} ({})", device.name, device.address);
        }
    }

    // Win32 windows (Windows only)
    #[cfg(target_os = "windows")]
    {
        println!("\n    Scanning for desktop windows...");
        match Toolkit::find_desktop_windows() {
            Ok(windows) => {
                if windows.is_empty() {
                    println!("    No window found.");
                } else {
                    for (i, win) in windows.iter().take(3).enumerate() {
                        println!("    [{}] hwnd={:?}, class={}", i, win.hwnd, win.class_name);
                    }
                    if windows.len() > 3 {
                        println!("    ... and {} more", windows.len() - 3);
                    }
                }
            }
            Err(e) => println!("    Failed to find windows: {}", e),
        }
    }

    // -------------------------------------------------------------------------
    // 3. Create Controller (choose one)
    // -------------------------------------------------------------------------
    println!("\n[3] Creating controller...");

    let controller: Option<Controller>;

    // Option A: ADB Controller
    #[cfg(feature = "adb")]
    if let Some(device) = adb_devices.first() {
        println!("    Using ADB device: {}", device.name);
        let config_str = serde_json::to_string(&device.config)?;
        let adb_path = device.adb_path.to_str().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid ADB path")
        })?;
        controller = Some(Controller::new_adb(
            adb_path,
            &device.address,
            &config_str,
            "",
        )?);
    } else {
        controller = None;
    }

    #[cfg(not(feature = "adb"))]
    {
        controller = None;
    }

    // Option B: Win32 Controller (uncomment to use)
    // #[cfg(target_os = "windows")]
    // if let Some(window) = windows.first() {
    //     controller = Some(Controller::new_win32(
    //         window.hwnd as isize,
    //         common::Win32ScreencapMethod::FramePool as i32,
    //         common::Win32InputMethod::PostMessage as i32,
    //         common::Win32InputMethod::PostMessage as i32,
    //     )?);
    // }

    // -------------------------------------------------------------------------
    // 4. Controller Operations
    // -------------------------------------------------------------------------
    if let Some(ref ctrl) = controller {
        println!("\n[4] Controller operations...");

        // Connect
        let conn_id = ctrl.post_connection()?;
        ctrl.wait(conn_id);
        println!("    Connected: {}", ctrl.connected());

        // Screenshot
        let cap_id = ctrl.post_screencap()?;
        ctrl.wait(cap_id);
        println!("    Screenshot captured");

        // Click (sync)
        let click_id = ctrl.post_click(114, 514)?;
        ctrl.wait(click_id);
        println!("    Clicked at (114, 514)");

        // Click (async) - post now, wait later
        let click_id2 = ctrl.post_click(514, 114)?;
        // ... do other work here ...
        ctrl.wait(click_id2);
        println!("    Async click completed");

        // Input text
        let input_id = ctrl.post_input_text("Hello MAA!")?;
        ctrl.wait(input_id);
        println!("    Text input sent");

        // Get resolution
        if let Ok((w, h)) = ctrl.resolution() {
            println!("    Resolution: {}x{}", w, h);
        }
    } else {
        println!("\n[4] Skipping controller operations (no device available)");
    }

    // -------------------------------------------------------------------------
    // 5. Resource Setup
    // -------------------------------------------------------------------------
    println!("\n[5] Setting up resource...");

    let resource = Resource::new()?;

    // Register custom recognition and action
    resource.register_custom_recognition("MyRecognition", Box::new(MyRecognition))?;
    resource.register_custom_action("MyAction", Box::new(MyAction))?;
    println!("    Registered: MyRecognition, MyAction");

    // List registered components
    let reco_list = resource.custom_recognition_list()?;
    let action_list = resource.custom_action_list()?;
    println!("    Recognition list: {:?}", reco_list);
    println!("    Action list: {:?}", action_list);

    // Load resource bundle (optional - commented out as sample resources don't exist)
    // Uncomment below if you have resource files
    let resource_path = "sample/resource";
    println!("    Loading resource from: {}", resource_path);
    let resource_loaded = match resource.post_bundle(resource_path) {
        Ok(job) => {
            let status = job.wait();
            println!("    Load status: {:?}", status);
            status == common::MaaStatus::SUCCEEDED
        }
        Err(e) => {
            println!("    Load failed: {} (OK for demo)", e);
            false
        }
    };

    if resource_loaded {
        println!("    ✅ Resource loaded successfully!");
    } else {
        println!("    ⚠️  Resource not loaded - OpenSettings task will be skipped");
    }

    // Add event sink
    resource.add_sink(|msg, details| {
        println!(
            "    [ResourceEvent] {}: {}",
            msg,
            &details[..details.len().min(50)]
        );
    })?;

    // -------------------------------------------------------------------------
    // 6. Tasker Setup and Execution
    // -------------------------------------------------------------------------
    println!("\n[6] Setting up tasker...");

    let tasker = Tasker::new()?;

    // Bind resource
    tasker.bind_resource(&resource)?;
    println!("    Resource bound");

    // Bind controller (if available)
    if let Some(ref ctrl) = controller {
        tasker.bind_controller(ctrl)?;
        println!("    Controller bound");
    }

    // Check initialization
    if tasker.inited() {
        println!("    Tasker initialized successfully!");

        // Add event sink
        tasker.add_sink(|msg, details| {
            println!(
                "    [TaskerEvent] {}: {}",
                msg,
                &details[..details.len().min(100)]
            );
        })?;

        // Execute task with pipeline override
        let pipeline_override = r#"{
            "MyCustomEntry": {
                "recognition": "Custom",
                "custom_recognition": "MyRecognition",
                "action": "Custom",
                "custom_action": "MyAction"
            }
        }"#;

        // Execute "OpenSettings" task (if resource was loaded and controller is available)
        if resource_loaded && controller.is_some() {
            println!("\n[7a] Opening Android Settings...");
            match tasker.post_task("OpenSettings", "{}") {
                Ok(job) => {
                    println!("    Task posted, waiting for completion...");
                    let status = job.wait();
                    println!("    OpenSettings task status: {:?}", status);
                    
                    if status == common::MaaStatus::SUCCEEDED {
                        println!("    ✅ Successfully opened Android Settings!");
                        
                        // Wait a moment for the app to fully open
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        
                        // Verify by checking screen
                        if let Some(ref ctrl) = controller {
                            println!("    Taking screenshot to verify...");
                            let cap_id = ctrl.post_screencap()?;
                            ctrl.wait(cap_id);
                        }
                    } else {
                        println!("    ⚠️  Task completed with status: {:?}", status);
                    }
                    
                    if let Ok(Some(detail)) = job.get(false) {
                        println!("    Entry: {}", detail.entry);
                        println!("    Nodes executed: {}", detail.nodes.len());
                    }
                }
                Err(e) => println!("    ❌ OpenSettings task failed: {}", e),
            }
        } else {
            println!("\n[7a] Skipping OpenSettings task:");
            if !resource_loaded {
                println!("    - Resource not loaded");
            }
            if controller.is_none() {
                println!("    - No controller available (need ADB device)");
            }
        }

        // println!("\n[7b] Executing custom task...");
        match tasker.post_task("MyCustomEntry", pipeline_override) {
            Ok(job) => {
                let status = job.wait();
                println!("    Task status: {:?}", status);

                if let Ok(Some(detail)) = job.get(false) {
                    println!("    Entry: {}", detail.entry);
                    println!("    Nodes executed: {}", detail.nodes.len());

                    for node_opt in detail.nodes {
                        if let Some(node) = node_opt {
                            if let Some(reco) = node.recognition {
                                println!(
                                    "      Node: {}, Algo: {:?}",
                                    node.node_name, reco.algorithm
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => println!("    Task failed: {}", e),
        }
    } else {
        println!("    Tasker not fully initialized (missing controller binding)");
    }

    // -------------------------------------------------------------------------
    // 8. Cleanup
    // -------------------------------------------------------------------------
    println!("\n[8] Demo completed!");
    println!("    - Toolkit, Controller, Resource, Tasker all demonstrated");
    println!("    - Custom Recognition and Action registered and explained");
    println!("    - Context API documented in custom component implementations");

    Ok(())
}

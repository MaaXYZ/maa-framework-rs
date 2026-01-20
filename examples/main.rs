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

use maa_framework::context::Context;
use maa_framework::controller::Controller;
use maa_framework::custom::{CustomAction, CustomRecognition};
use maa_framework::resource::Resource;
use maa_framework::tasker::Tasker;
use maa_framework::toolkit::Toolkit;
use maa_framework::{buffer, common, sys};

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
        _image: *const sys::MaaImageBuffer,
        _roi: &sys::MaaRect,
    ) -> Option<(sys::MaaRect, String)> {
        println!(
            "[MyRecognition] analyze called: node={}, name={}, param={}",
            node_name, custom_recognition_name, custom_recognition_param
        );

        // --- Context API Demo ---

        // 1. Run sub-pipeline recognition with override
        let pp_override = r#"{"MyCustomOCR": {"recognition": "OCR", "roi": [100, 100, 200, 300]}}"#;
        if let Ok(img_buf) = buffer::MaaImageBuffer::new() {
            let reco_id = context.run_recognition("MyCustomOCR", pp_override, &img_buf);
            println!("  run_recognition result: {:?}", reco_id);
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
                    sys::MaaRect {
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
        if let Ok(new_ctx) = context.clone_context() {
            let _ = new_ctx.override_pipeline(r#"{"MyCustomOCR": {"roi": [100, 200, 300, 400]}}"#);

            // Run recognition and use the result
            if let Ok(img_buf) = buffer::MaaImageBuffer::new() {
                let reco_id = new_ctx.run_recognition("MyCustomOCR", "{}", &img_buf);

                // If recognition succeeded, get the tasker to retrieve details
                if reco_id.is_ok() {
                    // In a real scenario, you would:
                    // 1. Get recognition detail from tasker
                    // 2. Check if it hit
                    // 3. Use the box coordinates to click
                    println!("  [Demo] Would get reco detail and click on result box");

                    // Example of clicking on recognition result box:
                    // if let Ok(Some(detail)) = tasker.get_recognition_detail(reco_id) {
                    //     if detail.hit {
                    //         let box_rect = detail.box_rect;
                    //         controller.post_click(box_rect.x, box_rect.y).wait();
                    //     }
                    // }
                }
            }
            // new_ctx changes don't affect original context
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
            sys::MaaRect {
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
        box_rect: &sys::MaaRect,
    ) -> bool {
        println!(
            "[MyAction] run called: node={}, name={}, param={}",
            node_name, custom_action_name, custom_action_param
        );
        println!(
            "  reco_id={}, box=({}, {}, {}, {})",
            reco_id, box_rect.x, box_rect.y, box_rect.width, box_rect.height
        );

        // You can use Context API here too
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

        true // Return true for success, false for failure
    }
}

// ============================================================================
// Main Demo
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MaaFramework Rust SDK Demo ===\n");

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
        controller = Some(Controller::new_adb(
            device.adb_path.to_str().unwrap(),
            &device.address,
            &config_str,
            None,
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

    // Load resource bundle
    let resource_path = "sample/resource";
    println!("    Loading resource from: {}", resource_path);
    match resource.post_bundle(resource_path) {
        Ok(job) => {
            let status = job.wait();
            println!("    Load status: {:?}", status);
        }
        Err(e) => println!("    Load failed: {} (OK for demo)", e),
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
                &details[..details.len().min(50)]
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

        println!("\n[7] Executing task...");
        match tasker.post_task("MyCustomEntry", pipeline_override) {
            Ok(job) => {
                let status = job.wait();
                println!("    Task status: {:?}", status);

                if let Ok(Some(detail)) = job.get(false) {
                    println!("    Entry: {}", detail.entry);
                    println!("    Nodes executed: {}", detail.node_id_list.len());
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

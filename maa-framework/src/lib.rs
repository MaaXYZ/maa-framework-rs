//! # MaaFramework Rust Bindings
//!
//! High-performance, safe Rust bindings for [MaaFramework](https://github.com/MaaXYZ/MaaFramework),
//! a game automation framework based on image recognition.
//!
//! ## Quick Start
//!
//! ```no_run
//! use maa_framework::toolkit::Toolkit;
//! use maa_framework::controller::Controller;
//! use maa_framework::resource::Resource;
//! use maa_framework::tasker::Tasker;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 0. Load library (Dynamic only)
//!     #[cfg(feature = "dynamic")]
//!     maa_framework::load_library(std::path::Path::new("MaaFramework.dll"))?;
//!
//!     // 1. Find devices
//!     let devices = Toolkit::find_adb_devices()?;
//!     let device = devices.first().expect("No device found");
//!
//!     // 2. Create controller (agent_path: "" to use MAA_AGENT_PATH or current dir)
//!     let adb_path = device.adb_path.to_str().ok_or_else(|| {
//!         std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid ADB path")
//!     })?;
//!     let controller = Controller::new_adb(adb_path, &device.address, "{}", "")?;
//!
//!     // 3. Create resource and tasker
//!     let resource = Resource::new()?;
//!     let tasker = Tasker::new()?;
//!
//!     // 4. Bind and run
//!     tasker.bind(&resource, &controller)?;
//!     let job = tasker.post_task("StartTask", "{}")?;
//!     job.wait();
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Core Modules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`tasker`] | Task execution and pipeline management |
//! | [`resource`] | Resource loading (images, models, pipelines) |
//! | [`controller`] | Device control (ADB, Win32, PlayCover) |
//! | [`context`] | Task execution context for custom components |
//! | [`toolkit`] | Device discovery utilities |
//! | [`pipeline`] | Pipeline configuration types |
//! | [`job`] | Asynchronous job management |
//! | [`event_sink`] | Event sink system for typed callbacks |
//! | [`buffer`] | Safe data buffers for FFI |
//! | [`custom`] | Custom recognizer and action traits |
//! | [`custom_controller`] | Custom controller implementation |
//! | [`notification`] | Structured event notification parsing |
//! | [`common`] | Common types and data structures |
//! | [`error`] | Error types and handling |
//! | [`util`] | Miscellaneous utility functions |
//! | [`agent_client`] | Remote custom component client |
//! | [`agent_server`] | Remote custom component server |
//!
//! ## Feature Flags
//!
//! - `adb` - ADB controller support (default)
//! - `win32` - Win32 controller support (Windows only)
//! - `custom` - Custom recognizer/action/controller support
//! - `toolkit` - Device discovery utilities
//! - `image` - Integration with the `image` crate

#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

pub mod agent_client;
pub mod agent_server;
pub mod buffer;
pub mod callback;
pub mod common;
pub mod context;
pub mod controller;
pub mod custom;
pub mod custom_controller;
pub mod error;
pub mod event_sink;
pub mod job;
pub mod notification;
pub mod pipeline;
pub mod resource;
pub mod tasker;
pub mod toolkit;
pub mod util;

pub use common::ControllerFeature;
pub use common::MaaStatus;
pub use error::{MaaError, MaaResult};

pub use maa_framework_sys as sys;

use std::ffi::CString;

/// Get the MaaFramework version string.
///
/// # Example
/// ```no_run
/// println!("MaaFramework version: {}", maa_framework::maa_version());
/// ```
pub fn maa_version() -> &'static str {
    unsafe {
        std::ffi::CStr::from_ptr(sys::MaaVersion())
            .to_str()
            .unwrap_or("unknown")
    }
}

/// Set a global framework option.
///
/// Low-level function for setting global options. Consider using the
/// convenience wrappers like [`configure_logging`], [`set_debug_mode`], etc.
pub fn set_global_option(
    key: sys::MaaGlobalOption,
    value: *mut std::ffi::c_void,
    size: u64,
) -> MaaResult<()> {
    let ret = unsafe { sys::MaaGlobalSetOption(key, value, size) };
    common::check_bool(ret)
}

/// Configure the log output directory.
///
/// # Arguments
/// * `log_dir` - Path to the directory where logs should be stored
pub fn configure_logging(log_dir: &str) -> MaaResult<()> {
    let c_dir = CString::new(log_dir)?;
    set_global_option(
        sys::MaaGlobalOptionEnum_MaaGlobalOption_LogDir as i32,
        c_dir.as_ptr() as *mut _,
        c_dir.as_bytes().len() as u64,
    )
}

/// Enable or disable debug mode.
///
/// In debug mode:
/// - Recognition details include raw images and draws
/// - All tasks are treated as focus tasks and produce callbacks
///
/// # Arguments
/// * `enable` - `true` to enable debug mode
pub fn set_debug_mode(enable: bool) -> MaaResult<()> {
    let mut val_bool = if enable { 1u8 } else { 0u8 };
    set_global_option(
        sys::MaaGlobalOptionEnum_MaaGlobalOption_DebugMode as i32,
        &mut val_bool as *mut _ as *mut _,
        std::mem::size_of::<u8>() as u64,
    )
}

/// Set the log level for stdout output.
///
/// # Arguments
/// * `level` - Logging level (use `sys::MaaLoggingLevel*` constants)
pub fn set_stdout_level(level: sys::MaaLoggingLevel) -> MaaResult<()> {
    let mut val = level;
    set_global_option(
        sys::MaaGlobalOptionEnum_MaaGlobalOption_StdoutLevel as i32,
        &mut val as *mut _ as *mut _,
        std::mem::size_of::<sys::MaaLoggingLevel>() as u64,
    )
}

/// Enable/disable saving recognition visualizations to log directory.
pub fn set_save_draw(enable: bool) -> MaaResult<()> {
    let mut val: u8 = if enable { 1 } else { 0 };
    set_global_option(
        sys::MaaGlobalOptionEnum_MaaGlobalOption_SaveDraw as i32,
        &mut val as *mut _ as *mut _,
        std::mem::size_of::<u8>() as u64,
    )
}

/// Enable/disable saving screenshots on error.
pub fn set_save_on_error(enable: bool) -> MaaResult<()> {
    let mut val: u8 = if enable { 1 } else { 0 };
    set_global_option(
        sys::MaaGlobalOptionEnum_MaaGlobalOption_SaveOnError as i32,
        &mut val as *mut _ as *mut _,
        std::mem::size_of::<u8>() as u64,
    )
}

/// Set JPEG quality for saved draw images (0-100, default 85).
pub fn set_draw_quality(quality: i32) -> MaaResult<()> {
    let mut val = quality;
    set_global_option(
        sys::MaaGlobalOptionEnum_MaaGlobalOption_DrawQuality as i32,
        &mut val as *mut _ as *mut _,
        std::mem::size_of::<i32>() as u64,
    )
}

/// Set the recognition image cache limit (default 4096).
pub fn set_reco_image_cache_limit(limit: u64) -> MaaResult<()> {
    let mut val = limit;
    set_global_option(
        sys::MaaGlobalOptionEnum_MaaGlobalOption_RecoImageCacheLimit as i32,
        &mut val as *mut _ as *mut _,
        std::mem::size_of::<u64>() as u64,
    )
}

/// Load a plugin from the specified path.
pub fn load_plugin(path: &str) -> MaaResult<()> {
    let c_path = CString::new(path)?;
    let ret = unsafe { sys::MaaGlobalLoadPlugin(c_path.as_ptr()) };
    common::check_bool(ret)
}

/// Loads the MaaFramework dynamic library.
///
/// You **must** call this function successfully before using any other APIs when the `dynamic`
/// feature is enabled.
///
/// # Arguments
///
/// * `path` - Path to the dynamic library file (e.g., `MaaFramework.dll`, `libMaaFramework.so`).
///
/// # Errors
///
/// Returns an error if:
/// * The library file cannot be found or loaded.
/// * The library has already been loaded (multiple initialization is not supported).
/// * Required symbols are missing from the library.
///
/// # Panics
///
/// Subsequent calls to any MaaFramework API will panic if the library has not been initialized.
///
/// # Safety
///
/// This function is `unsafe` because:
/// * It executes arbitrary initialization code (e.g., `DllMain`) inside the loaded library.
/// * The caller must ensure `path` points to a valid MaaFramework binary compatible with these bindings.
#[cfg(feature = "dynamic")]
pub fn load_library(path: &std::path::Path) -> Result<(), String> {
    unsafe { sys::load_library(path) }
}

/// Finds and loads the MaaFramework dynamic library when using the `dynamic` feature.
///
/// Tries, in order: `MAA_SDK_PATH` (bin/lib), project `MAA-*` dirs (from `CARGO_MANIFEST_DIR`
/// or current dir), `target/debug` or `target/release`, then current dir. Use this in examples
/// or apps so that `cargo run --example main` works without setting env vars.
///
/// # Errors
///
/// Returns an error if no library file is found or loading fails.
#[cfg(feature = "dynamic")]
pub fn ensure_library_loaded() -> Result<(), String> {
    let lib_name = if cfg!(target_os = "windows") {
        "MaaFramework.dll"
    } else if cfg!(target_os = "macos") {
        "libMaaFramework.dylib"
    } else {
        "libMaaFramework.so"
    };

    let mut candidates: Vec<std::path::PathBuf> = Vec::new();

    if let Ok(sdk) = std::env::var("MAA_SDK_PATH") {
        let sdk = std::path::PathBuf::from(sdk);
        candidates.push(sdk.join("bin").join(lib_name));
        candidates.push(sdk.join("lib").join(lib_name));
    }

    let search_roots: Vec<std::path::PathBuf> = std::env::var("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .into_iter()
        .chain(std::env::current_dir().ok())
        .collect();

    for root in &search_roots {
        if let Ok(entries) = std::fs::read_dir(root) {
            for e in entries.flatten() {
                let p = e.path();
                if p.is_dir() {
                    if let Some(name) = p.file_name() {
                        if name.to_string_lossy().starts_with("MAA-") {
                            candidates.push(p.join("bin").join(lib_name));
                        }
                    }
                }
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        candidates.push(cwd.join("target/debug").join(lib_name));
        candidates.push(cwd.join("target/release").join(lib_name));
        candidates.push(cwd.join(lib_name));
    }

    let chosen = candidates.into_iter().find(|p| p.exists());
    match chosen {
        Some(path) => load_library(&path),
        None => Err(
            "MaaFramework library not found. Set MAA_SDK_PATH or place SDK (e.g. MAA-*/bin/)."
                .to_string(),
        ),
    }
}

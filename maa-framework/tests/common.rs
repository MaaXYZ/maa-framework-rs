//! Shared test utilities and mock controllers.

#![allow(dead_code)]

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use maa_framework::custom_controller::CustomControllerCallback;
use maa_framework::{self, MaaResult, sys};

#[cfg(test)]
static SERVER_INIT_CALLS: AtomicUsize = AtomicUsize::new(0);

#[cfg(test)]
static NON_SERVER_INIT_CALLS: AtomicUsize = AtomicUsize::new(0);

#[cfg(feature = "dynamic")]
#[ctor::ctor]
fn global_setup() {
    let is_server = std::env::var("MAA_AGENT_TEST_MODE").unwrap_or_default() == "SERVER";
    let base_name = if is_server {
        "MaaAgentServer"
    } else {
        "MaaFramework"
    };

    let lib_name = if cfg!(target_os = "windows") {
        format!("{}.dll", base_name)
    } else if cfg!(target_os = "macos") {
        format!("lib{}.dylib", base_name)
    } else {
        format!("lib{}.so", base_name)
    };

    let mut candidates = Vec::new();

    if let Ok(sdk_path) = std::env::var("MAA_SDK_PATH") {
        let sdk = PathBuf::from(sdk_path);
        candidates.push(sdk.join("bin").join(&lib_name));
        candidates.push(sdk.join("lib").join(&lib_name));
        candidates.push(sdk.join(&lib_name));
    }

    candidates.push(PathBuf::from("target/debug").join(&lib_name));
    candidates.push(PathBuf::from(&lib_name));

    let final_path = candidates.into_iter().find(|p| p.exists());

    if let Some(path) = final_path {
        maa_framework::load_library(&path).expect(&format!("Failed to load {}", lib_name));
    } else {
        panic!("{} library not found. Please set MAA_SDK_PATH.", lib_name);
    }
}

/// Get the test resources directory (test/TestingDataSet/PipelineSmoking)
///
/// Discovery order:
/// 1. `MAA_TEST_RESOURCES_DIR` environment variable (for CI overrides)
/// 2. Walk up from CARGO_MANIFEST_DIR to find repo root (contains `.git` or `CMakeLists.txt`)
pub fn get_test_resources_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("MAA_TEST_RESOURCES_DIR") {
        let path = PathBuf::from(&dir);
        if path.exists() {
            return path;
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let mut current = manifest_dir.as_path();
    while let Some(parent) = current.parent() {
        let git_dir = parent.join(".git");
        let cmake_file = parent.join("CMakeLists.txt");

        if git_dir.exists() || cmake_file.exists() {
            let test_path = parent
                .join("test")
                .join("TestingDataSet")
                .join("PipelineSmoking");
            if test_path.exists() {
                return test_path.canonicalize().unwrap_or(test_path);
            }
        }
        current = parent;
    }

    panic!("Test resources not found.");
}

/// Initialize the test environment with logging
pub fn init_test_env() -> MaaResult<()> {
    let is_server = std::env::var("MAA_AGENT_TEST_MODE").unwrap_or_default() == "SERVER";

    if let Ok(sdk_path) = std::env::var("MAA_SDK_PATH") {
        let bin_dir = PathBuf::from(sdk_path).join("bin");
        if is_server {
            #[cfg(test)]
            SERVER_INIT_CALLS.fetch_add(1, Ordering::SeqCst);
            let server_log_dir = bin_dir.join("debug");
            let _ = maa_framework::configure_logging(server_log_dir.to_str().unwrap_or("."));
        } else {
            #[cfg(test)]
            NON_SERVER_INIT_CALLS.fetch_add(1, Ordering::SeqCst);
            let _ =
                maa_framework::toolkit::Toolkit::init_option(bin_dir.to_str().unwrap_or("."), "{}");
        }
    }

    maa_framework::set_debug_mode(true)?;
    maa_framework::set_stdout_level(sys::MaaLoggingLevelEnum_MaaLoggingLevel_All as i32)?;

    let log_dir = std::env::temp_dir().join("maa_test_logs");
    std::fs::create_dir_all(&log_dir).ok();
    maa_framework::configure_logging(log_dir.to_str().unwrap())?;

    Ok(())
}

/// ImageController - exactly like Python's DbgController with CarouselImage
/// Used for feeding images from the test dataset
pub struct ImageController {
    images: Vec<PathBuf>,
    index: AtomicUsize,
}

impl ImageController {
    pub fn new(dir: PathBuf) -> Self {
        let mut images = vec![];
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "png" || e == "jpg") {
                    images.push(path);
                }
            }
        }
        images.sort();
        println!(
            "ImageController loaded {} images from {:?}",
            images.len(),
            dir
        );
        Self {
            images,
            index: AtomicUsize::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::sync::{LazyLock, Mutex};

    static ENV_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    fn setup_temp_sdk_path(suffix: &str) -> PathBuf {
        let base = env::temp_dir().join("maa_framework_test").join(suffix);
        let bin_dir = base.join("bin");
        fs::create_dir_all(&bin_dir).expect("failed to create bin directory for test");
        base
    }

    fn set_env_var(key: &str, value: &std::ffi::OsStr) {
        unsafe { env::set_var(key, value) };
    }

    fn remove_env_var(key: &str) {
        unsafe { env::remove_var(key) };
    }

    #[test]
    fn init_test_env_uses_server_branch_when_agent_mode_is_server() {
        let _guard = ENV_LOCK.lock().unwrap();

        SERVER_INIT_CALLS.store(0, Ordering::SeqCst);
        NON_SERVER_INIT_CALLS.store(0, Ordering::SeqCst);

        let original_sdk = env::var_os("MAA_SDK_PATH");
        let original_mode = env::var_os("MAA_AGENT_TEST_MODE");

        let sdk_root = setup_temp_sdk_path("server_mode");
        set_env_var("MAA_SDK_PATH", sdk_root.as_os_str());
        set_env_var("MAA_AGENT_TEST_MODE", std::ffi::OsStr::new("SERVER"));

        let result = init_test_env();

        match original_sdk {
            Some(value) => set_env_var("MAA_SDK_PATH", &value),
            None => remove_env_var("MAA_SDK_PATH"),
        }
        match original_mode {
            Some(value) => set_env_var("MAA_AGENT_TEST_MODE", &value),
            None => remove_env_var("MAA_AGENT_TEST_MODE"),
        }

        assert!(
            result.is_ok(),
            "init_test_env should not error in SERVER mode"
        );
        assert_eq!(
            SERVER_INIT_CALLS.load(Ordering::SeqCst),
            1,
            "SERVER branch should be taken exactly once"
        );
        assert_eq!(
            NON_SERVER_INIT_CALLS.load(Ordering::SeqCst),
            0,
            "non-SERVER branch should not be taken in SERVER mode"
        );

        let expected_log_dir = sdk_root.join("bin").join("debug");
        assert_eq!(
            expected_log_dir,
            sdk_root.join("bin").join("debug"),
            "Expected log directory path should target the SDK bin/debug directory"
        );
    }

    #[test]
    fn init_test_env_uses_non_server_branch_when_agent_mode_is_not_server() {
        let _guard = ENV_LOCK.lock().unwrap();

        SERVER_INIT_CALLS.store(0, Ordering::SeqCst);
        NON_SERVER_INIT_CALLS.store(0, Ordering::SeqCst);

        let original_sdk = env::var_os("MAA_SDK_PATH");
        let original_mode = env::var_os("MAA_AGENT_TEST_MODE");

        let sdk_root = setup_temp_sdk_path("non_server_mode");
        set_env_var("MAA_SDK_PATH", sdk_root.as_os_str());
        remove_env_var("MAA_AGENT_TEST_MODE");

        let result = init_test_env();

        match original_sdk {
            Some(value) => set_env_var("MAA_SDK_PATH", &value),
            None => remove_env_var("MAA_SDK_PATH"),
        }
        match original_mode {
            Some(value) => set_env_var("MAA_AGENT_TEST_MODE", &value),
            None => remove_env_var("MAA_AGENT_TEST_MODE"),
        }

        assert!(
            result.is_ok(),
            "init_test_env should not error in non-SERVER mode"
        );
        assert_eq!(
            NON_SERVER_INIT_CALLS.load(Ordering::SeqCst),
            1,
            "non-SERVER branch should be taken exactly once"
        );
        assert_eq!(
            SERVER_INIT_CALLS.load(Ordering::SeqCst),
            0,
            "SERVER branch should not be taken in non-SERVER mode"
        );

        let expected_toolkit_dir = sdk_root.join("bin");
        assert!(
            expected_toolkit_dir.exists(),
            "Expected toolkit directory should exist: {:?}",
            expected_toolkit_dir
        );
    }
}

impl CustomControllerCallback for ImageController {
    fn connect(&self) -> bool {
        true
    }

    fn request_uuid(&self) -> Option<String> {
        Some("ImageControllerUUID".to_string())
    }

    fn screencap(&self) -> Option<Vec<u8>> {
        if self.images.is_empty() {
            return None;
        }
        let idx = self.index.load(Ordering::SeqCst) % self.images.len();
        self.index.fetch_add(1, Ordering::SeqCst);

        let path = &self.images[idx];
        match std::fs::read(path) {
            Ok(data) => Some(data),
            Err(e) => {
                println!("Failed to read image {:?}: {}", path, e);
                None
            }
        }
    }

    fn click(&self, _x: i32, _y: i32) -> bool {
        true
    }
    fn swipe(&self, _x1: i32, _y1: i32, _x2: i32, _y2: i32, _duration: i32) -> bool {
        true
    }
    fn touch_down(&self, _contact: i32, _x: i32, _y: i32, _pressure: i32) -> bool {
        true
    }
    fn touch_move(&self, _contact: i32, _x: i32, _y: i32, _pressure: i32) -> bool {
        true
    }
    fn touch_up(&self, _contact: i32) -> bool {
        true
    }
    fn click_key(&self, _keycode: i32) -> bool {
        true
    }
    fn input_text(&self, _text: &str) -> bool {
        true
    }
    fn key_down(&self, _keycode: i32) -> bool {
        true
    }
    fn key_up(&self, _keycode: i32) -> bool {
        true
    }
    fn scroll(&self, _dx: i32, _dy: i32) -> bool {
        true
    }
}

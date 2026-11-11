//! Device-free native ABI and dynamic compatibility checks.
//!
//! Run with `cargo test -p maa-framework --features dynamic --test ios_contract
//! -- --test-threads=1`. The Linux fixture requires `cc` and pthreads, but no SDK.
//! `CC` selects one compiler executable here; values containing arguments are unsupported.
#![cfg(all(feature = "dynamic", target_os = "linux"))]

use std::ffi::CStr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};

use maa_framework::common::{IosConnectionMethod, IosScreencapMethod};
use maa_framework::controller::Controller;
use maa_framework::toolkit::Toolkit;
use maa_framework::{MaaError, sys};
use serde::Serialize;
use serde_json::json;

struct FixtureDir(PathBuf);

impl Drop for FixtureDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn compile_fixture(dir: &Path, name: &str, omitted: Option<&str>) -> PathBuf {
    let path = dir.join(format!("lib{name}.so"));
    let mut compiler = Command::new(std::env::var_os("CC").unwrap_or_else(|| "cc".into()));
    compiler.args([
        "-std=c11",
        "-D_POSIX_C_SOURCE=200809L",
        "-shared",
        "-fPIC",
        "-pthread",
    ]);
    compiler.arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ios_contract.c"));
    if let Some(symbol) = omitted {
        compiler.arg(format!("-D{symbol}"));
    }
    let output = compiler
        .arg("-o")
        .arg(&path)
        .output()
        .expect("run C fixture compiler");
    assert!(
        output.status.success(),
        "C fixture build failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    path
}

#[test]
fn ios_contract() {
    let dir =
        FixtureDir(std::env::temp_dir().join(format!("maa-ios-contract-{}", std::process::id())));
    std::fs::create_dir_all(&dir.0).unwrap();
    let full = compile_fixture(&dir.0, "IosContract", None);
    for scenario in [
        "unloaded",
        "controller",
        "null_controller",
        "invalid_udid",
        "serialization",
        "discovery",
        "options",
        "cancellation",
        "empty",
        "find_failed",
        "null_list",
        "null_at",
        "null_name",
        "null_udid",
        "null_state",
        "null_connections",
        "null_warnings",
        "bad_connections",
        "bad_warnings",
        "connections_not_array",
        "warnings_not_array",
    ] {
        run_child(scenario, &full);
    }
    for (scenario, omitted) in [
        ("missing_list_destroy", "OMIT_LIST_DESTROY"),
        ("missing_state", "OMIT_GET_STATE"),
        ("missing_options", "OMIT_FIND_OPTIONS"),
        ("missing_controller", "OMIT_CONTROLLER_CREATE"),
        ("missing_controller_destroy", "OMIT_CONTROLLER_DESTROY"),
        ("missing_controller_clear", "OMIT_CONTROLLER_CLEAR"),
    ] {
        let path = compile_fixture(&dir.0, scenario, Some(omitted));
        run_child(scenario, &path);
    }
}

fn run_child(scenario: &str, path: &Path) {
    // Each process gets a fresh OnceCell and a private native library instance.
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "ios_contract_child", "--nocapture"])
        .env("MAA_IOS_CONTRACT_SCENARIO", scenario)
        .env("MAA_IOS_CONTRACT_LIBRARY", path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{scenario}:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

struct Mock(sys::CompositeLibrary);

impl Mock {
    fn count(&self, index: i32) -> u64 {
        unsafe {
            let f = self
                .0
                .get::<unsafe extern "C" fn(i32) -> u64>(b"MockCount\0")
                .unwrap();
            f(index)
        }
    }

    fn scenario(&self, value: i32) {
        unsafe {
            let f = self
                .0
                .get::<unsafe extern "C" fn(i32)>(b"MockSetScenario\0")
                .unwrap();
            f(value);
        }
    }

    fn text(&self, symbol: &[u8]) -> String {
        unsafe {
            let f = self
                .0
                .get::<unsafe extern "C" fn() -> *const std::ffi::c_char>(symbol)
                .unwrap();
            CStr::from_ptr(f()).to_str().unwrap().to_owned()
        }
    }

    fn connection(&self) -> i32 {
        unsafe {
            self.0
                .get::<unsafe extern "C" fn() -> i32>(b"MockLastConnection\0")
                .unwrap()()
        }
    }

    fn methods(&self) -> u64 {
        unsafe {
            self.0
                .get::<unsafe extern "C" fn() -> u64>(b"MockLastMethods\0")
                .unwrap()()
        }
    }
}

fn create_controller() -> maa_framework::MaaResult<Controller> {
    Controller::new_ios(
        "offline-udid",
        IosScreencapMethod::DEFAULT,
        IosConnectionMethod::Wifi,
        &json!({"server_port": 17423, "shared_control": true}),
    )
}

fn unavailable<T: std::fmt::Debug>(result: maa_framework::MaaResult<T>, symbol: &str) {
    match result.unwrap_err() {
        MaaError::ApiUnavailable(message) => assert!(message.contains(symbol), "{message}"),
        other => panic!("expected unavailable {symbol}, got {other:?}"),
    }
}

#[test]
fn ios_contract_child() {
    let Ok(scenario) = std::env::var("MAA_IOS_CONTRACT_SCENARIO") else {
        return;
    };
    if scenario == "unloaded" {
        assert!(matches!(
            Toolkit::find_ios_devices(),
            Err(MaaError::ApiUnavailable(_))
        ));
        assert!(matches!(
            create_controller(),
            Err(MaaError::ApiUnavailable(_))
        ));
        return;
    }
    let library = PathBuf::from(std::env::var_os("MAA_IOS_CONTRACT_LIBRARY").unwrap());
    let mock = Mock(unsafe { sys::CompositeLibrary::new(&library) }.unwrap());
    maa_framework::load_library(&library).unwrap();
    match scenario.as_str() {
        "controller" => {
            assert_eq!(IosConnectionMethod::Auto as i32, 0);
            assert_eq!(IosConnectionMethod::Usb as i32, 1);
            assert_eq!(IosConnectionMethod::Wifi as i32, 2);
            assert_eq!(IosScreencapMethod::NONE.bits(), 0);
            assert_eq!(IosScreencapMethod::STREAM.bits(), 1);
            assert_eq!(IosScreencapMethod::SCREENSHOT_SERVICE.bits(), 2);
            assert_eq!(IosScreencapMethod::ALL.bits(), u64::MAX);
            assert_eq!(IosScreencapMethod::DEFAULT.bits(), u64::MAX);
            let controller = create_controller().unwrap();
            assert_eq!(mock.connection(), 2);
            assert_eq!(mock.methods(), u64::MAX);
            assert_eq!(mock.text(b"MockLastUdid\0"), "offline-udid");
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&mock.text(b"MockLastConfig\0")).unwrap(),
                json!({"server_port":17423,"shared_control":true})
            );
            let clone = controller.clone();
            drop(controller);
            assert_eq!(mock.count(5), 0);
            assert_eq!(mock.count(9), 1);
            drop(clone);
            assert_eq!(mock.count(4), 1);
            assert_eq!(mock.count(5), 1);
            assert_eq!(mock.count(6), 1);
            assert_eq!(mock.count(9), 0);
        }
        "null_controller" => {
            mock.scenario(1);
            assert!(matches!(
                create_controller(),
                Err(MaaError::FrameworkError(-1))
            ));
            assert_eq!(mock.count(4), 1);
            assert_eq!(mock.count(5), 0);
        }
        "invalid_udid" => {
            for udid in ["", "invalid\0udid"] {
                assert!(
                    Controller::new_ios(
                        udid,
                        IosScreencapMethod::STREAM,
                        IosConnectionMethod::Auto,
                        &json!({})
                    )
                    .is_err()
                );
            }
            assert_eq!(mock.count(4), 0);
        }
        "serialization" => {
            struct Failing;
            impl Serialize for Failing {
                fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                    Err(serde::ser::Error::custom("fixture serialization failed"))
                }
            }
            assert!(
                Controller::new_ios(
                    "device",
                    IosScreencapMethod::STREAM,
                    IosConnectionMethod::Auto,
                    &Failing
                )
                .is_err()
            );
            assert!(
                Toolkit::find_ios_devices_with_options(IosConnectionMethod::Auto, &Failing, None)
                    .is_err()
            );
            assert_eq!(mock.count(0), 0);
            assert_eq!(mock.count(4), 0);
        }
        "discovery" => {
            let result = Toolkit::find_ios_devices().unwrap();
            assert_eq!(mock.count(0), 1);
            assert_eq!(mock.count(1), 1);
            assert_eq!(mock.count(8), 0);
            assert_eq!(mock.count(2), 1);
            let device = &result.devices[0];
            assert_eq!(device.name, "Test iPhone");
            assert_eq!(device.udid, "offline-udid");
            assert_eq!(device.state, "discovered");
            assert_eq!(
                device.connections,
                vec![json!({"transport":"usb","paired":false,"port":49152})]
            );
            assert_eq!(device.screencap_methods, 0x8000_0000_0000_0003);
            assert_eq!(result.warnings, ["offline fixture"]);
        }
        "options" => {
            Toolkit::find_ios_devices_with_options(
                IosConnectionMethod::Usb,
                &json!({"timeout_ms":123,"server_port":17424}),
                None,
            )
            .unwrap();
            assert_eq!(mock.connection(), 1);
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&mock.text(b"MockLastOptions\0"))
                    .unwrap(),
                json!({"timeout_ms":123,"server_port":17424})
            );
            assert_eq!(mock.count(2), 0);
            assert_eq!(mock.count(3), 1);
            assert_eq!(mock.count(1), 1);
            assert_eq!(mock.count(7), 0);
        }
        "cancellation" => {
            let cancel = AtomicBool::new(true);
            match Toolkit::find_ios_devices_with_options(
                IosConnectionMethod::Auto,
                &json!({}),
                Some(&cancel),
            )
            .unwrap_err()
            {
                MaaError::IosDiscoveryFailed { warnings } => assert_eq!(warnings, ["cancelled"]),
                other => panic!("unexpected cancellation error: {other:?}"),
            }
            assert_eq!(mock.count(7), 1);
            assert_eq!(mock.count(1), 1);
            cancel.store(false, Ordering::Relaxed);
            Toolkit::find_ios_devices_with_options(
                IosConnectionMethod::Auto,
                &json!({}),
                Some(&cancel),
            )
            .unwrap();
            assert_eq!(mock.count(7), 2);
            assert_eq!(mock.count(1), 2);
        }
        "find_failed" => {
            mock.scenario(3);
            match Toolkit::find_ios_devices().unwrap_err() {
                MaaError::IosDiscoveryFailed { warnings } => {
                    assert_eq!(warnings, ["offline fixture"])
                }
                other => panic!("unexpected discovery error: {other:?}"),
            }
            assert_eq!(mock.count(1), 1);
        }
        "empty" => {
            mock.scenario(10);
            let result = Toolkit::find_ios_devices().unwrap();
            assert!(result.devices.is_empty());
            assert_eq!(result.warnings, ["offline fixture"]);
            assert_eq!(mock.count(1), 1);
        }
        "missing_list_destroy" | "missing_state" => {
            let symbol = if scenario == "missing_list_destroy" {
                "MaaToolkitIOSDeviceListDestroy"
            } else {
                "MaaToolkitIOSDeviceGetState"
            };
            unavailable(Toolkit::find_ios_devices(), symbol);
            assert_eq!(
                mock.count(0),
                0,
                "preflight must finish before allocating a list"
            );
        }
        "missing_options" => {
            Toolkit::find_ios_devices().unwrap();
            unavailable(
                Toolkit::find_ios_devices_with_options(IosConnectionMethod::Auto, &json!({}), None),
                "MaaToolkitIOSDeviceFindWithOptions",
            );
            assert_eq!(mock.count(0), 1);
            assert_eq!(mock.count(1), 1);
        }
        "missing_controller" | "missing_controller_destroy" | "missing_controller_clear" => {
            let symbol = match scenario.as_str() {
                "missing_controller" => "MaaIOSControllerCreate",
                "missing_controller_destroy" => "MaaControllerDestroy",
                _ => "MaaControllerClearSinks",
            };
            unavailable(create_controller(), symbol);
            assert_eq!(
                mock.count(4),
                0,
                "preflight must finish before creating a controller"
            );
            // Missing controller support must not disable independent toolkit APIs.
            Toolkit::find_ios_devices().unwrap();
        }
        _ => {
            let value = match scenario.as_str() {
                "null_list" => 2,
                "null_at" => 4,
                "null_name" => 5,
                "bad_connections" => 6,
                "bad_warnings" => 7,
                "connections_not_array" => 8,
                "warnings_not_array" => 9,
                "null_udid" => 11,
                "null_state" => 12,
                "null_connections" => 13,
                "null_warnings" => 14,
                _ => panic!("unknown fixture scenario {scenario}"),
            };
            mock.scenario(value);
            let error = Toolkit::find_ios_devices().unwrap_err();
            if scenario.starts_with("null_") {
                assert!(matches!(error, MaaError::NullPointer), "{error:?}");
            } else {
                assert!(matches!(error, MaaError::JsonError(_)), "{error:?}");
            }
            assert_eq!(mock.count(0), 1);
            assert_eq!(mock.count(1), u64::from(scenario != "null_list"));
            assert_eq!(mock.count(8), 0);
        }
    }
}

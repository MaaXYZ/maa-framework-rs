//! Run with `cargo test -p maa-framework --features dynamic --test ios_optional_symbols
//! -- --test-threads=1`. Requires Linux and a C compiler, without an SDK or device.
//! `CC`, when set, selects a single compiler executable.
#![cfg(all(feature = "dynamic", target_os = "linux"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use maa_framework::controller::Controller;
use maa_framework::toolkit::Toolkit;
use maa_framework::{IosConnectionMethod, IosScreencapMethod, MaaError, MaaResult, sys};

const CONTROLLER: &[&str] = &[
    "MaaIOSControllerCreate",
    "MaaControllerClearSinks",
    "MaaControllerDestroy",
];
const DISCOVERY: &[&str] = &[
    "MaaToolkitIOSDeviceListCreate",
    "MaaToolkitIOSDeviceListDestroy",
    "MaaToolkitIOSDeviceListSize",
    "MaaToolkitIOSDeviceListAt",
    "MaaToolkitIOSDeviceGetName",
    "MaaToolkitIOSDeviceGetUdid",
    "MaaToolkitIOSDeviceGetState",
    "MaaToolkitIOSDeviceGetConnections",
    "MaaToolkitIOSDeviceGetScreencapMethods",
    "MaaToolkitIOSDeviceListGetWarnings",
    "MaaToolkitIOSDeviceFind",
    "MaaToolkitIOSDeviceFindWithOptions",
];

struct FixtureDir(PathBuf);

impl Drop for FixtureDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn controller() -> MaaResult<Controller> {
    Controller::new_ios(
        "offline-udid",
        IosScreencapMethod::DEFAULT,
        IosConnectionMethod::Auto,
        &serde_json::json!({}),
    )
}

fn options() -> MaaResult<Vec<maa_framework::toolkit::IosDevice>> {
    Toolkit::find_ios_devices_with_options(IosConnectionMethod::Auto, &serde_json::json!({}), None)
}

fn unavailable<T>(result: MaaResult<T>, symbol: &str) {
    match result {
        Err(MaaError::ApiUnavailable(missing)) => assert_eq!(missing, symbol),
        Err(error) => panic!("expected unavailable {symbol}, got {error:?}"),
        Ok(_) => panic!("missing API {symbol} must return an error"),
    }
}

fn child(scenario: &str, path: &Path) {
    if scenario == "unloaded" {
        unavailable(controller(), "<library not loaded>");
        unavailable(Toolkit::find_ios_devices(), "<library not loaded>");
        unavailable(options(), "<library not loaded>");
        return;
    }

    maa_framework::load_library(path).unwrap();
    assert_eq!(maa_framework::maa_version(), "5.15.0");
    if scenario == "old" {
        unavailable(controller(), "MaaIOSControllerCreate");
        unavailable(Toolkit::find_ios_devices(), "MaaToolkitIOSDeviceFind");
        unavailable(options(), "MaaToolkitIOSDeviceFindWithOptions");
        return;
    }

    let mock = unsafe { sys::CompositeLibrary::new(path) }.unwrap();
    let count =
        |symbol: &[u8]| unsafe { mock.get::<unsafe extern "C" fn() -> u64>(symbol).unwrap()() };
    if scenario == "present" {
        let handle = controller().unwrap();
        let clone = handle.clone();
        drop(handle);
        assert_eq!(count(b"MockLiveHandles\0"), 1);
        drop(clone);
        assert_eq!(Toolkit::find_ios_devices().unwrap()[0].udid, "offline-udid");
        assert_eq!(options().unwrap()[0].name, "fixture");
        assert_eq!(count(b"MockAllocations\0"), 3);
    } else if CONTROLLER.contains(&scenario) {
        unavailable(controller(), scenario);
        assert_eq!(count(b"MockAllocations\0"), 0);
        assert_eq!(Toolkit::find_ios_devices().unwrap().len(), 1);
    } else {
        let expected = if scenario == "double_missing" {
            "MaaToolkitIOSDeviceListDestroy"
        } else {
            scenario
        };
        if scenario != "MaaToolkitIOSDeviceFindWithOptions" {
            unavailable(Toolkit::find_ios_devices(), expected);
        }
        if scenario != "MaaToolkitIOSDeviceFind" {
            unavailable(options(), expected);
        }
        assert_eq!(
            count(b"MockAllocations\0"),
            0,
            "preflight must precede allocation"
        );
        // Each discovery entry point remains usable independently of the other.
        if scenario == "MaaToolkitIOSDeviceFind" {
            assert_eq!(options().unwrap().len(), 1);
        } else if scenario == "MaaToolkitIOSDeviceFindWithOptions" {
            assert_eq!(Toolkit::find_ios_devices().unwrap().len(), 1);
        }
    }
    assert_eq!(count(b"MockLiveHandles\0"), 0);
}

#[test]
fn ios_optional_symbols() {
    if let Ok(scenario) = std::env::var("MAA_IOS_OPTIONAL_SCENARIO") {
        child(
            &scenario,
            &PathBuf::from(std::env::var_os("MAA_IOS_OPTIONAL_LIBRARY").unwrap()),
        );
        return;
    }

    let dir =
        FixtureDir(std::env::temp_dir().join(format!("maa-ios-optional-{}", std::process::id())));
    std::fs::create_dir_all(&dir.0).unwrap();
    let mut failures = Vec::new();
    for scenario in ["unloaded", "old", "present", "double_missing"]
        .into_iter()
        .chain(CONTROLLER.iter().copied())
        .chain(DISCOVERY.iter().copied())
    {
        let path = dir.0.join(format!("lib{scenario}.so"));
        if scenario != "unloaded" {
            let mut compiler = Command::new(std::env::var_os("CC").unwrap_or_else(|| "cc".into()));
            compiler.args(["-shared", "-fPIC"]).arg(
                Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ios_optional_symbols.c"),
            );
            if scenario == "old" {
                compiler.arg("-DOLD_SDK");
            } else if scenario == "double_missing" {
                compiler.args([
                    "-DMaaToolkitIOSDeviceListDestroy=MissingDestroy",
                    "-DMaaToolkitIOSDeviceGetName=MissingName",
                ]);
            } else if scenario != "present" {
                compiler.arg(format!("-D{scenario}=MissingSymbol"));
            }
            let output = compiler.arg("-o").arg(&path).output().unwrap();
            assert!(
                output.status.success(),
                "{scenario}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        // Each subprocess gets a fresh OnceCell; aborts must fail the parent test.
        let output = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "ios_optional_symbols", "--nocapture"])
            .env("MAA_IOS_OPTIONAL_SCENARIO", scenario)
            .env("MAA_IOS_OPTIONAL_LIBRARY", &path)
            .current_dir(&dir.0)
            .output()
            .unwrap();
        if !output.status.success() {
            let summary = String::from_utf8_lossy(&output.stderr)
                .lines()
                .take(8)
                .collect::<Vec<_>>()
                .join("\n");
            failures.push(format!("{scenario}: {}\n{summary}", output.status));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

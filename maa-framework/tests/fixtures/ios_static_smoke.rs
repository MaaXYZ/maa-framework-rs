//! Standalone consumer for the static binding path, linked to ios_contract.c.
//! Build only in a temporary Cargo project whose maa-framework path dependency
//! enables `static` and whose MAA_SDK_PATH points to that fixture's libraries.

use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;

use maa_framework::MaaError;
use maa_framework::common::{IosConnectionMethod, IosScreencapMethod};
use maa_framework::controller::Controller;
use maa_framework::toolkit::Toolkit;

#[link(name = "MaaFramework")]
unsafe extern "C" {
    fn MockCount(index: i32) -> u64;
    fn MockSetScenario(value: i32);
    fn MockLastConnection() -> i32;
    fn MockLastMethods() -> u64;
}

fn count(index: i32) -> u64 {
    unsafe { MockCount(index) }
}

fn scenario(value: i32) {
    unsafe { MockSetScenario(value) }
}

fn main() {
    let options = BTreeMap::<String, u32>::new();
    let controller = Controller::new_ios(
        "offline-udid",
        IosScreencapMethod::DEFAULT,
        IosConnectionMethod::Wifi,
        &options,
    )
    .unwrap();
    assert_eq!(unsafe { MockLastConnection() }, 2);
    assert_eq!(unsafe { MockLastMethods() }, u64::MAX);
    let clone = controller.clone();
    drop(controller);
    assert_eq!(count(5), 0);
    drop(clone);
    assert_eq!(count(5), 1);
    assert_eq!(count(6), 1);
    assert_eq!(count(9), 0);

    scenario(1);
    assert!(matches!(
        Controller::new_ios(
            "offline-udid",
            IosScreencapMethod::STREAM,
            IosConnectionMethod::Auto,
            &options,
        ),
        Err(MaaError::FrameworkError(-1))
    ));
    assert_eq!(count(5), 1);

    scenario(0);
    let result = Toolkit::find_ios_devices().unwrap();
    assert_eq!(count(1), 1);
    assert_eq!(count(8), 0);
    assert_eq!(result.devices[0].name, "Test iPhone");
    assert_eq!(result.devices[0].udid, "offline-udid");
    assert_eq!(result.devices[0].state, "discovered");
    assert_eq!(result.devices[0].connections.len(), 1);
    assert_eq!(result.devices[0].screencap_methods, 0x8000_0000_0000_0003);
    assert_eq!(result.warnings, ["offline fixture"]);

    scenario(3);
    match Toolkit::find_ios_devices().unwrap_err() {
        MaaError::IosDiscoveryFailed { warnings } => assert_eq!(warnings, ["offline fixture"]),
        other => panic!("unexpected discovery error: {other:?}"),
    }
    assert_eq!(count(1), 2);

    scenario(6);
    assert!(matches!(
        Toolkit::find_ios_devices(),
        Err(MaaError::JsonError(_))
    ));
    assert_eq!(count(1), 3);

    scenario(0);
    let cancel = AtomicBool::new(true);
    match Toolkit::find_ios_devices_with_options(IosConnectionMethod::Usb, &options, Some(&cancel))
        .unwrap_err()
    {
        MaaError::IosDiscoveryFailed { warnings } => assert_eq!(warnings, ["cancelled"]),
        other => panic!("unexpected cancellation error: {other:?}"),
    }
    assert_eq!(unsafe { MockLastConnection() }, 1);
    assert_eq!(count(7), 1);
    assert_eq!(count(1), 4);

    scenario(2);
    assert!(matches!(
        Toolkit::find_ios_devices(),
        Err(MaaError::NullPointer)
    ));
    assert_eq!(count(0), 5);
    assert_eq!(count(1), 4);
    assert_eq!(count(8), 0);
    println!("static iOS ABI, ownership, error, and worker cancellation smoke checks passed");
}

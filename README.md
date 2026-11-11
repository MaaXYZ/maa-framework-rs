<!-- markdownlint-disable MD033 MD041 -->
<p align="center">
  <img alt="LOGO" src="https://cdn.jsdelivr.net/gh/MaaAssistantArknights/design@main/v1/icons/maa-logo_512x512.png" width="256" height="256" />
</p>

<h1 align="center">MaaFramework Rust Binding</h1>

<div align="center">
  <div>
    <a href="https://github.com/MaaXYZ/maa-framework-rs/blob/main/LICENSE.md">
      <img alt="license" src="https://img.shields.io/github/license/MaaXYZ/maa-framework-rs">
    </a>
    <a href="https://crates.io/crates/maa-framework">
      <img alt="crates.io" src="https://img.shields.io/crates/v/maa-framework">
    </a>
    <a href="https://docs.rs/maa-framework">
      <img alt="docs.rs" src="https://img.shields.io/docsrs/maa-framework">
    </a>
  </div>
  <div>
    <a href="https://github.com/MaaXYZ/MaaFramework/releases/latest">
      <img alt="maa framework" src="https://img.shields.io/github/v/release/MaaXYZ/MaaFramework?label=MaaFramework">
    </a>
  </div>
</div>

<br />

<p align="center">
  English | <a href="https://github.com/MaaXYZ/maa-framework-rs/blob/main/README_zh.md">简体中文</a>
</p>

<p align="center">
Rust bindings for <a href="https://github.com/MaaXYZ/MaaFramework">MaaFramework</a>, a next-generation automation framework based on image recognition.
</p>

## ✨ Features

- **Idiomatic Rust** - Ergonomic safe wrappers with `Result` error handling and strict typing.
- **Dual Linking Modes** - Choose between **Static** linking or runtime **Dynamic** loading with automatic SDK setup.
- **Full Capabilities** - Complete coverage of Tasker pipelines, Resources, Controllers, and custom extensions.
- **Zero-Overhead FFI** - Direct bindings via `bindgen` ensuring C++ level performance with Rust safety.

## 📦 Installation

### 1. Add Dependency

```toml
[dependencies]
maa-framework = "1"
```

### 2. Download SDK

Download from [MaaFramework Releases](https://github.com/MaaXYZ/MaaFramework/releases/latest):

| Platform | Architecture | Download |
| -------- | ------------ | -------- |
| Windows  | x86_64       | `MAA-win-x86_64-*.zip` |
| Windows  | aarch64      | `MAA-win-aarch64-*.zip` |
| Linux    | x86_64       | `MAA-linux-x86_64-*.zip` |
| Linux    | aarch64      | `MAA-linux-aarch64-*.zip` |
| macOS    | x86_64       | `MAA-macos-x86_64-*.zip` |
| macOS    | aarch64      | `MAA-macos-aarch64-*.zip` |

### 3. Extract to Project

```
my-project/
├── Cargo.toml
├── src/
│   └── main.rs
└── MAA-win-x86_64-v5.4.1/    # Extracted SDK
    ├── bin/
    ├── lib/
    └── include/
```

Or set `MAA_SDK_PATH` environment variable.

### 4. Build & Run

```bash
cargo build
cargo run
```

> DLLs are automatically copied to `target/debug/` or `target/release/`.

## 🔗 Linking Modes

MaaFramework Rust Binding supports two linking modes: **Static** (default) and **Dynamic**.

### Static Linking (Default)

- **Initialization**: Automatic. No extra code required.
- **Behavior**: The OS loader handles DLL loading at startup.
- **Requirement**: `MaaFramework.dll` (or `.so`/`.dylib`) must be in the system search path (usually next to the executable).
- **Usage**:
  ```toml
  [dependencies]
  maa-framework = "1"
  ```

### Dynamic Linking

- **Initialization**: Manual. You must call `load_library` before using any API.
- **Behavior**: Your code loads the DLL at runtime from a custom path.
- **Requirement**: A valid path to the DLL file.
- **Usage**:
  ```toml
  [dependencies]
  maa-framework = { version = "1", features = ["dynamic"] }
  ```
  And in your code:
  ```rust
  // Must be called before any other API
  maa_framework::load_library(std::path::Path::new("path/to/MaaFramework.dll"))?;
  ```

## 🚀 Quick Start

```rust
use maa_framework::toolkit::Toolkit;
use maa_framework::controller::Controller;
use maa_framework::resource::Resource;
use maa_framework::tasker::Tasker;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // [Dynamic only] Load MaaFramework library
    #[cfg(feature = "dynamic")]
    maa_framework::load_library(std::path::Path::new("MaaFramework.dll"))?;

    Toolkit::init_option("./", "{}")?;

    let devices = Toolkit::find_adb_devices()?;
    if devices.is_empty() {
        eprintln!("No ADB device found");
        return Ok(());
    }

    let device = &devices[0];
    let controller = Controller::new_adb(
        device.adb_path.to_str().unwrap(),
        &device.address,
        &device.config.to_string(),
        None,
    )?;
    controller.post_connection()?;

    let resource = Resource::new()?;
    resource.post_bundle("./resource")?;

    let tasker = Tasker::new()?;
    tasker.bind_controller(&controller)?;
    tasker.bind_resource(&resource)?;

    if !tasker.inited() {
        eprintln!("Failed to initialize MAA");
        return Ok(());
    }

    tasker.post_task("Startup", "{}")?;
    println!("Task started!");

    Ok(())
}
```

## iOS devices

`Controller::new_ios` and `Toolkit::find_ios_devices` support physical iOS devices
through XDeviceBridge. These APIs first appeared in MaaFramework **5.16.0-beta.1**;
5.15 SDKs do not provide them. The Rust crate version does not establish native
SDK compatibility.

```rust
use maa_framework::{IosConnectionMethod, IosScreencapMethod};
use maa_framework::{controller::Controller, toolkit::Toolkit};

fn example() -> maa_framework::MaaResult<()> {
    let result = Toolkit::find_ios_devices()?;
    for warning in &result.warnings {
        eprintln!("{warning}");
    }
    if let Some(device) = result.devices.first() {
        let controller = Controller::new_ios(
            &device.udid,
            IosScreencapMethod::DEFAULT,
            IosConnectionMethod::Auto,
            &serde_json::json!({}),
        )?;
        let connection = controller.post_connection()?;
        let status = controller.wait(connection);
        println!("Connection status: {status:?}");
    }
    Ok(())
}
```

`find_ios_devices_with_options` accepts a connection method, serializable options
(`timeout_ms`, `server_port`) and an optional borrowed `AtomicBool` cancellation
flag. Results own their strings and transport JSON. Discovery failure/cancellation
returns `MaaError::IosDiscoveryFailed` with warnings. Discovery records and a
successful constructor do not establish a working device connection.

Dynamic loading permits older SDKs to load; these new safe APIs return
`MaaError::ApiUnavailable` if a required symbol is missing, including a cleanup
symbol. Raw `sys` shims retain their existing panic behavior. Static mode requires
the iOS symbols when linking code that uses them; regenerating bindings requires
5.16 headers. Both modes still need the iOS control unit, `xdb_client` and a running
XDeviceBridge Server 1.6.0 or later. Missing backend support can return a null controller or
failed discovery even when all symbols are present. `max_bitrate` and `max_fps`
controller options require xdb Server 1.6.0 or later.

The Linux device-free FFI contract suite uses a small C fixture:

```bash
cargo test -p maa-framework --features dynamic --test ios_contract -- --test-threads=1
python3 maa-framework/tests/fixtures/run_ios_static_smoke.py
```

These tests check ABI, ownership, cancellation and missing-symbol handling; they
do not validate discovery, pairing, input or capture on real devices.

## 🔧 Features

| Feature | Description | Default |
|---------|-------------|---------|
| `toolkit` | Device discovery utilities | ✅ |
| `adb` | ADB controller support | ✅ |
| `win32` | Win32 controller (Windows) | ✅ |
| `custom` | Custom recognizer/action | ✅ |
| `image` | `image` crate integration | ❌ |

## 📚 Documentation

- [API Documentation](https://docs.rs/maa-framework)
- [MaaFramework Docs](https://github.com/MaaXYZ/MaaFramework/tree/main/docs)

## 📄 License

LGPL-3.0 - see [LICENSE](LICENSE.md)

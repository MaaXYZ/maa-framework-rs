<!-- markdownlint-disable MD033 MD041 -->
<p align="center">
  <img alt="LOGO" src="https://cdn.jsdelivr.net/gh/MaaAssistantArknights/design@main/logo/maa-logo_512x512.png" width="256" height="256" />
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

Rust bindings for [MaaFramework](https://github.com/MaaXYZ/MaaFramework), a next-generation automation framework based on image recognition.

## ✨ Features

- **Full API Coverage** - Complete bindings for MaaFramework APIs
- **Safe Rust** - Memory-safe wrappers with proper lifetime management
- **DLL Auto-Copy** - Runtime libraries are copied to `target/` automatically

## 📦 Installation

### 1. Add Dependency

```toml
[dependencies]
maa-framework = "0.6"
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

## 🚀 Quick Start

```rust
use maa_framework::toolkit::Toolkit;
use maa_framework::controller::Controller;
use maa_framework::resource::Resource;
use maa_framework::tasker::Tasker;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

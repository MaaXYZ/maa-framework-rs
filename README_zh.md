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
  <a href="https://github.com/MaaXYZ/maa-framework-rs/blob/main/README.md">English</a> | 简体中文
</p>

[MaaFramework](https://github.com/MaaXYZ/MaaFramework) 的 Rust 绑定，基于图像识别的新一代自动化框架。

## ✨ 特性

- **Rust 原生体验** - 符合人体工程学的安全封装，依托 `Result` 错误处理与强类型枚举。
- **双模链接支持** - 按需选择 **静态链接** 或 **动态加载**，自动处理 SDK 路径与 DLL 复制。
- **全能力覆盖** - 完整支持 Tasker 需要的任务流水线、资源加载、控制器及自定义组件扩展。
- **零开销 FFI** - 基于 `bindgen` 生成的直接绑定，在享受 Rust 安全性的同时保持 C++ 级性能。

## 📦 安装

### 1. 添加依赖

```toml
[dependencies]
maa-framework = "0.6"
```

### 2. 下载 SDK

从 [MaaFramework Releases](https://github.com/MaaXYZ/MaaFramework/releases/latest) 下载：

| 平台 | 架构 | 下载 |
| ---- | ---- | ---- |
| Windows | x86_64 | `MAA-win-x86_64-*.zip` |
| Windows | aarch64 | `MAA-win-aarch64-*.zip` |
| Linux | x86_64 | `MAA-linux-x86_64-*.zip` |
| Linux | aarch64 | `MAA-linux-aarch64-*.zip` |
| macOS | x86_64 | `MAA-macos-x86_64-*.zip` |
| macOS | aarch64 | `MAA-macos-aarch64-*.zip` |

### 3. 解压到项目

```
my-project/
├── Cargo.toml
├── src/
│   └── main.rs
└── MAA-win-x86_64-v5.4.1/    # 解压的 SDK
    ├── bin/
    ├── lib/
    └── include/
```

或设置 `MAA_SDK_PATH` 环境变量。

### 4. 构建运行

```bash
cargo build
cargo run
```

> DLL 会自动复制到 `target/debug/` 或 `target/release/`。

## 🔗 链接模式

MaaFramework Rust Binding 支持两种链接模式：**静态链接** (默认) 和 **动态链接**。

### 静态链接 (Static - 默认)

- **初始化**: 自动。无需额外代码。
- **行为**: 操作系统在程序启动时自动加载 DLL。
- **要求**: `MaaFramework.dll` (或 `.so`/`.dylib`) 必须在系统搜索路径中（通常与可执行文件同级）。
- **用法**:
  ```toml
  [dependencies]
  maa-framework = "0.6"
  ```

### 动态链接 (Dynamic)

- **初始化**: 手动。必须在使用任何 API 之前调用 `load_library`。
- **行为**: 代码在运行时从指定路径加载 DLL。
- **要求**: 指向 DLL 文件的有效路径。
- **用法**:
  ```toml
  [dependencies]
  maa-framework = { version = "0.6", features = ["dynamic"] }
  ```
  在代码中：
  ```rust
  // 必须在调用任何其他 API 之前调用
  maa_framework::load_library(std::path::Path::new("path/to/MaaFramework.dll"))?;
  ```

## 🚀 快速开始

```rust
use maa_framework::toolkit::Toolkit;
use maa_framework::controller::Controller;
use maa_framework::resource::Resource;
use maa_framework::tasker::Tasker;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // [仅动态链接模式] 加载 MaaFramework 库
    #[cfg(feature = "dynamic")]
    maa_framework::load_library(std::path::Path::new("MaaFramework.dll"))?;

    Toolkit::init_option("./", "{}")?;

    let devices = Toolkit::find_adb_devices()?;
    if devices.is_empty() {
        eprintln!("未找到 ADB 设备");
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
        eprintln!("MAA 初始化失败");
        return Ok(());
    }

    tasker.post_task("Startup", "{}")?;
    println!("任务已启动!");

    Ok(())
}
```

## 🔧 Features

| Feature | 描述 | 默认 |
|---------|------|------|
| `toolkit` | 设备发现工具 | ✅ |
| `adb` | ADB 控制器支持 | ✅ |
| `win32` | Win32 控制器 (Windows) | ✅ |
| `custom` | 自定义识别器/动作 | ✅ |
| `image` | `image` crate 集成 | ❌ |

## 📚 文档

- [API 文档](https://docs.rs/maa-framework)
- [MaaFramework 文档](https://github.com/MaaXYZ/MaaFramework/tree/main/docs)

## 📄 License

LGPL-3.0 - 见 [LICENSE](LICENSE.md)

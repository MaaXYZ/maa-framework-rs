# 开发者贡献指南 (Contributing Guidelines)

欢迎参与 maa-framework-rs 项目！

本仓库是针对 MAA (MaaAssistantArknights) 底层 C/C++ 核心库的 Rust 绑定与封装。为了确保开发环境的一致性并顺利运行项目，请仔细阅读以下指南。

## 1. 开发前准备

### 1.1 关键依赖说明 (重要)
本项目**包含对 `MaaDbgControlUnit` (Debug Controller) 的绑定代码**。
由于官方发布的 MAA Release 包（以及大部分预编译包）通常**不包含**此组件，直接使用可能导致编译链接失败或运行时错误，且无法通过完整测试。因此，您**必须**手动从源码编译 MaaFramework 核心库并开启对应选项。

### 1.2 环境要求
*   **Rust Toolchain**: 推荐使用最新 Stable 版本。
*   **构建工具**: CMake (>= 3.20), Python 3 (用于下载 MAA 依赖), C++ 编译器 (MSVC/GCC/Clang)。
*   **MaaFramework Core**: 需手动编译。

### 1.3 构建 MaaFramework 核心库
请参考以下步骤准备开发环境（也可参考 `.github/workflows/test.yml`）：

1.  **克隆 MaaFramework**:
    ```bash
    git clone --recursive https://github.com/MaaXYZ/MaaFramework.git
    cd MaaFramework
    ```

2.  **下载第三方依赖**:
    ```bash
    python tools/maadeps-download.py
    ```

3.  **编译安装 (必须开启 Debug Controller)**:
    在配置 CMake 时，务必添加 `-DWITH_DBG_CONTROLLER=ON` 参数。

    ```bash
    # 示例 (Linux/macOS):
    cmake -B build -DWITH_DBG_CONTROLLER=ON -DCMAKE_INSTALL_PREFIX=../maa-install
    cmake --build build -j
    cmake --install build
    ```

4.  **设置环境变量**:
    将 `MAA_SDK_PATH` 设置为上一步的安装目录（包含 `bin`, `lib` 的目录），以便 Rust 能够找到编译好的库。
    ```bash
    export MAA_SDK_PATH=/path/to/your/maa-install
    ```

## 2. 架构说明
项目分为两层，请确保您的修改位于正确的 Crate 中：

*   **`maa-framework-sys`** (Unsafe):
    *   仅包含 `bindgen` 生成的 C 接口和必要的 FFI 类型定义。
    *   **原则上不包含** 任何业务逻辑或高级封装。
*   **`maa-framework`** (Safe):
    *   提供符合 Rust 习惯的安全抽象（RAII、Result 错误处理等）。
    *   这是开发者主要贡献的地方。

## 3. 核心编码规范

### 3.1 工程哲学与代码质量 (Crucial)
*   **Rust 惯用原则 (Idiomatic Rust)**：优先使用 Rust 标准库 Trait (如 `From`, `AsRef`, `Drop`) 实现功能。避免引入非 Rust 风格（如过度封装的 Getter/Setter）的设计模式。
*   **简约法则**：**如无必要，勿增实体**。我们追求解决问题的**最短路径**。代码设计必须精炼、直观，**严禁**过度工程 (Over-engineering) 和无意义的抽象层级。
*   **工具优先**：在尝试引入复杂的胶水代码或 Hack（例如手动解析/修改生成代码）之前，**必须**先充分调研现有工具链（如 `bindgen` 配置、`cargo` 特性）的原生支持。不要为了解决简单问题而引入不可维护的复杂系统。

### 3.2 Unsafe 代码
虽然我们不希望用繁琐的规则束缚开发者，但在涉及 FFI 和内存安全时，请遵循以下底线：

*   **最小化使用**：仅在调用底层 C API 或处理裸指针时使用 `unsafe`。
*   **安全性说明**：对于逻辑复杂的 `unsafe` 块（特别是涉及指针偏移、生命周期转换的代码），**强烈建议**在代码上方添加注释说明为什么这样做是安全的。
    ```rust
    // SAFETY: ptr 来源于 CString，保证非空且以 null 结尾
    unsafe { sys::SomeCApi(ptr) };
    ```
*   **资源释放**：确保所有从 C 侧获取的资源都实现了 `Drop` trait 以自动释放内存。

### 3.3 错误处理
*   **Result 优先**：不要在库代码中使用 `unwrap()` 或 `expect()`，除非你能 100% 确定该操作绝不会失败（例如静态已知的转换）。所有运行时错误都应通过 `MaaResult` 向上传播。
*   **Panic 隔离**：Rust 的 panic 跨越 FFI 边界是未定义行为（UB）。如果在回调函数中调用了用户闭包，请确保使用 `std::panic::catch_unwind` 捕获异常。

### 3.4 代码风格
我们建议在提交前运行标准格式化工具。虽然 CI 目前主要检查编译和测试，但保持风格一致有助于代码审查。

```bash
# 格式化代码
cargo fmt
```

## 4. 测试与验证

本项目的 CI 覆盖了 Windows, Linux 和 macOS 平台。为了节省您的时间，请在提交 PR 前在本地进行测试。

*   **运行测试**:
    确保您已按照第 1 节正确设置了 `MAA_SDK_PATH`。

    **注意**：必须强制单线程运行。并行测试会导致资源竞争，可能出现未知错误导致测试不通过。

    ```bash
    # 静态链接模式（默认）
    cargo test -- --test-threads=1

    # 动态链接模式（需要配置动态库路径）
    cargo test --features dynamic -- --test-threads=1
    ```
*   **新增功能**：如果您添加了新 API，请尽量在 `tests/` 下添加对应的集成测试，或在文档注释中添加 Example 代码。

## 5. 提交规范 (Commit Convention)

我们遵循 [Conventional Commits](https://www.conventionalcommits.org/) 规范，这有助于自动生成 Changelog。

**格式**: `<type>(<scope>): <subject>`

**常见 Type**:
*   `feat`: 新功能
*   `fix`: 修复 Bug
*   `docs`: 文档变更
*   `style`: 代码格式调整（不影响逻辑）
*   `refactor`: 代码重构
*   `test`: 测试用例变更
*   `chore`: 构建过程或辅助工具变更

**示例**:
*   `feat(context): add support for custom node hit count`
*   `fix(controller): resolve null pointer crash in new_adb`
*   `docs: update README with usage examples`

## 6. Pull Request 流程

1.  **Fork & Clone**: Fork 本仓库并克隆到本地。
2.  **创建分支**: 建议基于 `main` 分支创建功能分支。
3.  **提交代码**: 确保编译通过，测试覆盖。
4.  **发起 PR**: 描述您的变更内容。
5.  **Code Review**: 维护者会进行 Review。如果现有代码风格与指南有出入，以保持现有风格一致性为准。

感谢您的贡献！

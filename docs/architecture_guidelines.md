# Vectomancy 架构与开发规范 (Architecture & Guidelines)

本文档确立 `Vectomancy` 项目的底层架构规范与工程协同标准。本项目致力于打造工业级的命令行工具，确保代码基严格遵循 Clean Code（整洁代码）原则、Hexagonal Architecture（六边形架构，即端口与适配器模式），并完全兼容 Linux XDG Base Directory 规范，同时配合现代化的 Git 版本控制与 DevOps 交付策略进行演进。

本项目采用 **MIT License** 开源许可证，以最大化促进社区生态的二次开发与集成。

## 1. XDG Base Directory 规范深度集成

为了保持用户 `$HOME` 目录的整洁（避免 Dotfile 污染），并在遵循 systemd 用户空间标准的同时兼容 Windows/macOS 系统行为，本项目严禁使用硬编码的 `~/.vectomancy` 目录。统一依托 `directories` crate 动态解析 XDG 路径。

### 1.1 路径规划、用途与数据持久化策略

通过 `ProjectDirs::from("com", "vectomancy", "vectomancy")` 获取全局安全的基准路径，并实施明确的数据分级存储策略：

- **Config Dir (配置目录)**: `$XDG_CONFIG_HOME/vectomancy/` (通常为 `~/.config/vectomancy/`)

  - **用途**: 存储应用级配置文件 `config.toml`。

  - **设计细节**: 利用 `serde` 结合 `figment` 或 `config-rs` 库实现配置层的合并。系统将合并默认配置、用户 XDG 配置与环境变量。例如，用户可在此预设 `default_terms = 1000` 或 `tolerance = 0.5`，从而精简日常 CLI 调用参数。

  - **示例结构**:

    ```
    [math_engine]
    default_mode = "fourier"
    max_fft_terms = 5000
    
    [parser]
    rdp_tolerance = 1.2
    ```

- **Data Dir (数据目录)**: `$XDG_DATA_HOME/vectomancy/` (通常为 `~/.local/share/vectomancy/`)

  - **用途**: 存储非易失性用户资源，主要为用户自定义的 Tera 模板文件（如 `custom_p5js.tera`）。
  - **扩展性**: 系统在初始化 Template Engine 时，会优先扫描此目录。如果发现与二进制内置同名的模板（如 `python.tera`），则执行用户覆写（Override）策略。

- **Cache Dir (缓存目录)**: `$XDG_CACHE_HOME/vectomancy/` (通常为 `~/.cache/vectomancy/`)

  - **用途**: 存储运行时产生的高成本、易失性中间数据，加速二次执行（热重载）。
  - **缓存对象**:
    1. 超大图像经过 TSP 规划后的坐标序列矩阵 (`.bin` 格式，利用 `bincode` 极速反序列化)。
    2. `rustfft` 的 Planner 预计算状态。
  - **失效策略 (Eviction Policy)**: 缓存文件名应基于原始图像文件的绝对路径与文件内容摘要（如 SHA-256 Hash）共同生成。一旦原图修改，Hash 不匹配，旧缓存自动被系统丢弃并触发重新计算。

### 1.2 XDG 初始化层代码模式

初始化模块应具备自愈能力（Self-healing），在目录缺失时自动创建，在权限不足时优雅降级或抛出明确的 IO 错误：

```
use directories::ProjectDirs;
use std::fs;
use anyhow::{Context, Result};

pub struct AppDirs {
    pub config: std::path::PathBuf,
    pub data: std::path::PathBuf,
    pub cache: std::path::PathBuf,
}

pub fn init_xdg_dirs() -> Result<AppDirs> {
    let proj_dirs = ProjectDirs::from("com", "vectomancy", "vectomancy")
        .context("Could not determine XDG base directories for the current OS environment")?;

    let dirs = AppDirs {
        config: proj_dirs.config_dir().to_path_buf(),
        data: proj_dirs.data_dir().to_path_buf(),
        cache: proj_dirs.cache_dir().to_path_buf(),
    };

    // 确保基础架构目录存在，赋予适当的文件系统权限
    fs::create_dir_all(&dirs.config).context("Failed to create XDG Config directory")?;
    fs::create_dir_all(&dirs.data).context("Failed to create XDG Data directory")?;
    fs::create_dir_all(&dirs.cache).context("Failed to create XDG Cache directory")?;

    Ok(dirs)
}
```

## 2. 工程协同规范与 DevOps 基础设施

作为高质量开源项目，必须依赖规范的 Git 管理流程、声明式的环境构建策略以及 CI/CD 自动化门禁。

### 2.1 分支管理模型 (Branching Strategy)

采用轻量级的 **GitHub Flow** 变体模型，杜绝过度复杂的 Git Flow：

- `main`: 主分支，必须保持随时可编译、可发布状态（Deployable）。严禁直接 Push 提交到此分支。
- `feature/<ticket-or-name>`: 功能分支。例如 `feature/add-julia-template`。从 `main` 迁出，开发完成后通过 Pull Request (PR) 合并。
- `fix/<bug-name>`: 修复分支。用于处理 issue 追踪器中的 Bug。

### 2.2 约定式提交规范 (Conventional Commits)

每一条 Git Commit Message 必须遵循语义化规范格式，便于后期结合 GitHub Actions 自动生成 `CHANGELOG.md` 及触发 Semantic Release：

格式：`<type>(<scope>): <subject>`

- `feat(math)`: 新增功能。
- `fix(parser)`: 修复 Bug。
- `refactor(emitter)`: 代码重构。
- `chore(ci)`: 构建过程或辅助工具的变动。

### 2.3 开发环境一致性保证 (Reproducibility)

为消除 "It works on my machine" 的环境差异问题，全面引入以下声明式配置：

1. **Toolchain 锁定 (`rust-toolchain.toml`)**: 锁定 Rust 编译器版本（如 `1.75.0` 或 `nightly-202X-XX-XX`）及所需组件（`rustfmt`, `clippy`）。所有开发者及 CI 环境将共享同一编译器基线。
2. **格式化契约 (`.rustfmt.toml`)**: 统一定义最大行宽、导入排序、宏格式化等代码风格细节。
3. **Nix 声明式环境 (`flake.nix`)**: 面向 Nix/NixOS 用户的终极解决方案。提供完全纯净（Pure）、确定性的开发 Shell（通过 `nix develop` 自动获取系统级依赖如 `cmake`, `pkg-config`, `fontconfig` 以及 Rust 工具链），并支持构建 Nix Derivation。

### 2.4 持续集成与交付部署 (CI/CD & Containerization)

- **GitHub Actions (`.github/workflows/`)**:

  - `ci.yml`: 拦截器。任何指向 `main` 的 PR 都会触发自动化的 `fmt` 检查、`clippy` Linter 扫描以及 `cargo test` 单元测试。
  - `release.yml`: 交付流水线。当推送 Tag 时，使用 `cross` 执行跨平台编译 (Linux x86_64/aarch64, Windows, macOS)，并将静态链接的二进制产物自动发布至 GitHub Releases。

- **容器化交付 (`Dockerfile`)**:

  采用 Multi-stage Build（多阶段构建）技术：

  - Build Stage：基于 `rust:alpine` 或 `rust:slim` 编译 Release 二进制文件，确保隔离系统依赖。
  - Runtime Stage：将编译好的二进制文件 COPY 到 `scratch` 或极小体积的基础镜像中，输出仅数 MB 的微型容器，便于未来部署为 Web API Server 或在沙盒环境中执行批处理任务。

## 3. Clean Code 与 SOLID 实践指南

系统代码将严格遵循 Domain-Driven Design (DDD) 的思想，解耦核心业务逻辑 (Domain Core) 与 I/O 副作用 (Adapters)。

### 3.1 单一职责原则 (SRP) 的物理隔离

- **分离 I/O 与计算核心**: `src/math/` 模块下的任何函数**绝对禁止**包含 `println!`、文件读写操作或依赖外部环境变量。所有数学模块必须是高度确定性的纯函数（Pure Functions）或仅依赖传入不可变引用的闭包。
- CLI 解析（`clap`）、配置文件读取、日志输出以及模板渲染（`tera`）均视为边缘的 **Adapter (适配器) 层**，它们仅负责翻译外界请求并递交给 Domain，绝不参与核心算法规则。

### 3.2 依赖倒置原则 (DIP) - Traits 驱动架构

为支持多种数学拟合策略，通过抽象 `MathEngine` Trait 实行面向接口编程：

```
use crate::models::{Point2D, MathExpressionAST, VectomancyError};

/// 核心域：数学引擎行为规范
pub trait MathEngine {
    fn process(&self, points: &[Point2D]) -> Result<MathExpressionAST, VectomancyError>;
}
```

## 4. 错误处理与日志追踪 (Error & Tracing)

### 4.1 核心库层级：语义化错误 (thiserror)

使用 `thiserror` 定义统一的全局 Domain 错误枚举。利用 `#[from]` 属性实现低层（如 std::io::Error）向高层业务错误的自动类型提升与上下文封装：

```
use thiserror::Error;

#[derive(Error, Debug)]
pub enum VectomancyError {
    #[error("File system I/O failure: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Image preprocessing failed: No closed contours found.")]
    NoContourFound,
}
```

### 4.2 高级结构化诊断观测 (tracing)

全面拥抱 `tracing` 诊断框架，支持 **Span 级别的遥测 (Telemetry)**。这对于监控 TSP 等高耗时图论算法极具价值，结合 `tracing-subscriber` 配合 `-vv` 可输出精确的性能剖析火焰图数据。

## 5. 标准化工程目录全景 (Clean Directory Layout)

整合 XDG 环境隔离、Clean Code 架构划分、Git 配置以及 DevOps 基础设施后，本项目最终的 Rust 工程目录树拓扑结构如下：

```
vectomancy/
├── .github/                  # GitHub Actions 自动化工作流与 Issue 模板
│   └── workflows/
│       ├── ci.yml            # 持续集成门禁 (Lint, Test, Build)
│       └── release.yml       # 持续交付 (交叉编译并发布 GitHub Release Binary)
├── .git/                     # 版本控制系统底层元数据
├── .gitignore                # 忽略 target/, .env 等构建产物与临时文件
├── .pre-commit-config.yaml   # Git pre-commit hooks 配置 (本地触发 fmt/clippy)
├── .rustfmt.toml             # Rust 代码格式化一致性规范约束
├── rust-toolchain.toml       # 锁定编译器环境 (Channel, Components)
├── flake.nix                 # 纯净的 NixOS 开发环境声明与 Derivation 构建配置
├── Dockerfile                # 多阶段 (Multi-stage) 微型容器构建脚本
├── LICENSE                   # MIT 许可证授权声明
├── Cargo.toml                # Rust 依赖声明与 Workspace 配置
├── build.rs                  # 编译期脚本 (执行 Tera 默认模板的嵌入打包)
├── benches/                  # Criterion 性能基准测试目录 (压测 TSP/FFT)
├── tests/                    # 端到端 (E2E) 集成测试套件
├── default_templates/        # 随源码分发的基础 Tera 模板库
└── src/
    ├── main.rs               # [CLI边缘] 进程入口，注册 Tracing Subscriber，拼装执行图
    ├── cli.rs                # [Adapter] 基于 clap 定义命令行参数与子命令
    ├── config.rs             # [Adapter] XDG Config 目录 TOML 文件序列化与合并解析
    ├── error.rs              # [Domain] 全局错误定义 (thiserror 语义化声明)
    ├── models/               # [Domain] 核心限界上下文模型 (Point2D, MathExpressionAST)
    ├── parser/               # [Adapter] 多态输入源解析适配器 (Raster/Vector)
    ├── math/                 # [Domain] 核心数学与图论算子 (纯函数引擎 TSP/FFT/RDP)
    └── emitter/              # [Adapter] 跨平台渲染与输出发射器 (Tera)
```
# Vectomancy 系统工作流 (System Workflow)

本文档定义了 Vectomancy CLI 工具在运行时的核心执行流水线 (Execution Pipeline) 与数据流转机制。开发者应严格按照此工作流进行模块划分与代码实现。

## 1. 核心执行流水线 (Core Execution Pipeline)

系统执行过程分为 5 个具有严格先后依赖关系的状态阶段 (Phases)。

### Phase 1: 初始化与参数解析 (Initialization & CLI Parsing)

- **模块**: `src/cli.rs`
- **执行逻辑**:
  1. 触发 `clap` 解析用户传入的命令行参数 (`--mode`, `--terms`, `--format`, `--output`)。
  2. 初始化 `env_logger`，设置全局日志级别 (Info/Debug/Trace)。
  3. 校验输入文件路径 (File I/O Check)，识别文件扩展名并进入对应的多态处理分支。

### Phase 2: 多态输入预处理 (Polymorphic Preprocessing)

- **模块**: `src/parser/`
- **分支 A: Vector Mode (输入为 `.svg` 且未强制指定 fourier)**
  1. 调用 `usvg` 将 SVG DOM 解析为展平的几何路径 (Flattened Paths)。
  2. 提取 `BezierSegment` 集合。
  3. **数据流出**: `Vec<BezierSegment>`。
- **分支 B: Raster Mode (输入为 `.png/.jpg`)**
  1. 调用 `image` crate 加载位图至内存。
  2. 执行 `Grayscale` (灰度化) -> `Sobel Edge Detection` (边缘检测) -> `Otsu Binarization` (二值化) -> `Zhang-Suen Thinning` (骨架化) -> `RDP Reduction` (降采样)。
  3. Raster 模式现在支持 Fourier 和 Spline 两种模式。
  4. 若为 Spline 模式，将降采样后的点转换为 `BezierSegments`。
  5. 若为 Fourier 模式，触发 **TSP 路径连通 (2-Opt Nearest Neighbor)**，将离散点排序为一维连续时间序列。
  6. **数据流出**: `Ordered Point Signal: Vec<Complex64>` (Fourier) 或 `Vec<BezierSegment>` (Spline)。

### Phase 3: 数学引擎转换 (Mathematical Engine)

- **模块**: `src/math/`
- **执行逻辑**: 消费 Phase 2 产出的数据，进行纯数学变换。
  - **Spline Builder**: 接收来自 SVG 或 Raster 模式的 `BezierSegment` 集合，将其直接代数化，提取多项式系数。
  - **FFT Solver**: 调用 `rustfft` 对 `Ordered Point Signal` 执行离散傅里叶变换。按振幅 (Amplitude) 降序排列，截断至用户指定的 `--terms N` 项。
- **统一抽象**: 将上述两种策略的结果，统一封装为内部标准数据结构 `MathExpressionAST`（数学表达式抽象语法树）。

### Phase 4: 模板渲染发射 (Template Emission)

- **模块**: `src/emitter/`
- **执行逻辑**:
  1. 实例化 `Tera` 模板引擎，加载 `templates/` 目录下的所有预置模板 (`.tera`)。
  2. 根据 CLI 的 `--format` 参数选择对应的模板 (如 `python.tera`, `desmos.tera`)。
  3. 将 `MathExpressionAST` 序列化为 Context 并注入模板。
  4. 执行内存渲染，捕获渲染错误 (Template Syntax Error)。

### Phase 5: 终态输出 (Output & Teardown)

- **模块**: `src/main.rs`
- **执行逻辑**:
  1. 将渲染后的字符串执行 Disk I/O，写入用户指定的 `--output` 路径。
  2. 在 Terminal 打印成功日志及耗时统计 (Execution Time Metrics)。
  3. 释放内存，安全退出 (Exit Code 0)。

## 2. 状态机流转图 (State Machine Diagram)

以下是底层数据结构流转的抽象状态图：

```
+-------------------+      +-----------------------+      +-------------------------+
|  User CLI Input   | ---> |  File Type Detection  | ---> |   Raster / Vector AST   |
+-------------------+      +-----------------------+      +-------------------------+
                                                                     |
                                  +----------------------------------+
                                  |
                                  v
+-------------------+      +-----------------------+      +-------------------------+
| Ordered Time Ser. | <--- | TSP & RDP Optimizer   | <--- |   Point Cloud Extract   | (Raster Only)
+-------------------+      +-----------------------+      +-------------------------+
         |
         v
+-------------------+      +-----------------------+      +-------------------------+
| FFT Computation   | ---> | MathExpressionAST     | <--- |   Bezier Extraction     | (Vector Only)
+-------------------+      +-----------------------+      +-------------------------+
                                  |
                                  v
+-------------------+      +-----------------------+      +-------------------------+
| Tera Context Inj. | ---> |  Target .tera Parse   | ---> | Disk I/O (Final Source) |
+-------------------+      +-----------------------+      +-------------------------+
```

## 3. 开发者实施工作流 (Developer Implementation Workflow)

作为软件工程实践，建议您按照以下 Milestone 顺序推进项目：

1. **Milestone 1 (Scaffolding)**:
   - 初始化 Cargo 项目，引入 `clap`，搭建 CLI 骨架。确保所有 flag 和 subcommands 解析正确。
2. **Milestone 2 (The Raster Core)**:
   - 引入 `image` 和 `rustfft`。
   - 硬编码一张极简黑白图片（如空心圆），跑通 `提取像素 -> 假定有序 -> FFT -> 打印高频系数` 的控制台逻辑。
3. **Milestone 3 (The TSP Bottleneck)**:
   - 引入 RDP 算法与 Nearest Neighbor 算法。此为性能瓶颈点，建议编写 Benchmark 测试。
4. **Milestone 4 (The Emitter)**:
   - 引入 `tera`。在 `templates/` 建立 `latex.tera` 和 `python.tera`。
   - 实现 AST 到 Context 的序列化，完成真正的端到端 (End-to-End) 文件输出。
5. **Milestone 5 (Refactor & Vector)**:
   - 重构代码，提取模块。
   - 引入 SVG 解析链路，补齐精确贝塞尔参数化功能。

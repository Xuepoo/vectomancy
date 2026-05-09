# 软件需求规格说明书 (SRS) - Image-to-Equation Converter

## 1. 项目概述 (Project Overview)

### 1.1 项目基本信息

**项目名称**: `Vectomancy` (矢量魔法)

**项目目标**: 开发一个极具性能优势的命令行工具 (CLI)，将各类图形文件（包括传统光栅图与矢量图）深度解析，并转换为可在多语言、多平台绘图引擎中无缝展示的数学参数方程集合与可执行渲染脚本。

**技术生态**: 基于 Rust 语言构建核心底层引擎，采用模板驱动（Template-Driven）架构以支持极强的输出扩展性与解耦开发。

### 1.2 背景与动机 (Background & Motivation)

在计算机图形学与纯数学可视化之间，往往存在一道壁垒。生成复杂的数学方程（如包含几千个项的傅里叶级数）来描绘特定人物或图形（类似 Wolfram Alpha 的 "Person Curves"）通常需要复杂的定制化脚本。`Vectomancy` 旨在打破这一壁垒，提供一个标准化、开箱即用的系统级工具，让“图像到数学公式”的转换过程如同使用 FFmpeg 转换视频格式一样简单、高效且高度可定制。

### 1.3 核心愿景 (Core Vision)

成为极客艺术、数学教育演示以及跨端参数化绘图领域的**标准 CLI 构建工具**。通过极低的学习成本和极高的执行效率，让毫无图形学背景的用户也能瞬间生成令人惊叹的数学曲线代码。

## 2. 目标用户与使用场景 (Target Audience & Use Cases)

### 2.1 目标用户画像

- **算法研究员/数据科学家**: 需要在学术论文或报告中，使用纯代码框架（如 Python 的 Matplotlib 或 R 的 ggplot2）直接内联绘制特定轮廓图，以保持矢量精度和图表风格的一致性。
- **数学/信号处理教育工作者**: 在教授“傅里叶变换”、“周期函数”或“参数方程”等课程时，利用此工具生成学生熟悉的图像方程（如动漫角色轮廓），作为直观且引人入胜的教学案例。
- **生成艺术创作者 (Generative Artists)**: 探索参数化设计，将现实图像转化为纯粹的数学波形，并在此基础上叠加噪声或数学变异，创造具有科技感的数学艺术品。
- **前端与动画工程师**: 需要将复杂的 SVG 图标转换为纯函数驱动的 Web 动画（如利用 Canvas 或 p5.js），从而摆脱对外部图像文件的依赖。

### 2.2 典型使用场景示例

- **场景一：科研绘图内联化**

  用户输入一张 `.png` 格式的实验仪器简图，`Vectomancy` 输出一段 Python 脚本。用户将该脚本直接粘贴入 Jupyter Notebook，即可使用 `matplotlib` 完美重绘该仪器，且可任意缩放不失真。

- **场景二：动态数学演示**

  教师输入一张具有教育意义的符号图像，工具生成一段包含数百个三角函数的长文本。教师将其导入 GeoGebra 或 Desmos 画板，不仅能展示静态结果，还能通过调整时间参数 $t$ 演示曲线随时间逐渐闭合的动态绘画过程。

- **场景三：激光雕刻与数控路径预演**

  将轮廓转换为连续的一维参数方程后，进一步离散化处理，可作为轻量级的 CNC (数控机床) 或激光雕刻机的连续一笔画加工路径参考。

## 3. 核心功能需求 (Functional Requirements)

### 3.1 多模态输入与预处理模块 (Input Parsing & Preprocessing)

系统必须具备强大的图像包容性，支持并规范化处理两种维度的输入：

- **F-IN-01: 矢量图深度解析 (Vector Input)**
  - **格式支持**: 完整支持 `.svg` 格式文件的加载。
  - **路径提取**: 深度解析 SVG `<path>` 标签中的 `d` 属性，精确识别绝对与相对坐标，并提取 `M` (Move), `L` (Line), `C` (Cubic Bezier), `Q` (Quadratic Bezier), `A` (Arc), `Z` (Close) 等核心指令。
  - **属性展平 (Flattening)**: 自动处理 SVG 树状结构中的 `transform` 属性（如 `translate`, `scale`, `rotate`, `matrix`），将所有局部坐标系下的点统一映射至全局绝对坐标系。
  - **形状转换**: 将基础形状标签（如 `<rect>`, `<circle>`, `<polygon>`）在内存中自动转化为等效的 `<path>` 描述，统一后续处理流水线。
- **F-IN-02: 光栅图增强预处理 (Raster Input)**
  - **格式支持**: 支持 `.png`, `.jpg`, `.jpeg`, `.webp` 甚至包含 Alpha 通道的透明图片。
  - **降噪与二值化**: 内置高斯模糊 (Gaussian Blur) 滤镜消除图像噪点；采用 Otsu 自适应阈值算法将彩色/灰度图像转化为高质量的黑白二值图。
  - **轮廓追踪与细化**: 使用 Moore-Neighbor 追踪算法提取外边缘；使用 Sobel 边缘检测算法提取轮廓；对粗线条执行 Zhang-Suen 骨架细化操作，确保提取的边界为单像素宽度。
  - **降采样算法**: 强制应用 Ramer-Douglas-Peucker (RDP) 算法简化路径，大幅压缩点云规模，为后续图论计算减负。

### 3.2 混合数学转换引擎 (Mathematical Conversion Engine)

核心算法层，提供两种截然不同的数学拟合策略，用户可通过 CLI 参数 `--mode` 自由切换：

- **F-MATH-01: 精确贝塞尔参数化 (Exact Parametric Splines)**
  - **触发条件**: 默认模式。适用于矢量 SVG 输入，或将光栅图骨架化后转换为 Spline 路径。
  - **处理逻辑**: 遵循解析几何原理，将路径分段映射为参数方程组。通过将光栅图骨架转换为 Spline 路径，实现与矢量图一致的高精度拟合。
    - _直线段_: 转化为 $x(t) = (1-t)x_0 + t x_1, y(t) = (1-t)y_0 + t y_1$。
    - _高阶贝塞尔_: 输出标准的多项式参数方程。例如三次贝塞尔：$X(t) = (1-t)^3P_{0x} + 3t(1-t)^2P_{1x} + 3t^2(1-t)P_{2x} + t^3P_{3x}$，并严格限制定义域 $t \in [0, 1]$。
  - **优势**: 数学上 100% 精确，方程项数较少，计算速度极快。
- **F-MATH-02: 傅里叶级数一笔画逼近 (Fourier Series Approximation)**
  - **触发条件**: 显式指定 `--mode fourier` 时作为备选方案。
  - **路径连通 (TSP Optimization)**: 将分散的像素点群视为图论节点。运行带有 2-Opt 启发式优化的最近邻 (Nearest Neighbor) TSP（旅行商问题）算法，找到一条无交叉或少交叉的最优一笔画遍历路径，使之成为一个随时间 $t$ 变化的周期性一维信号。
  - **阶跃函数掩码**: 在处理非连通的图像部件（如两只分离的眼睛）时，在跃迁路径区间自动生成 Heaviside 阶跃函数 $\theta(x)$ 掩码，确保在数学渲染时该跃迁轨迹处于“隐形”状态。
  - **频域变换 (FFT)**: 将空间坐标映射为复数 $z(t) = x(t) + iy(t)$，执行快速傅里叶变换。用户可通过 `-n, --terms` 参数指定保留的最大振幅谐波数量（如 `-n 1000`）。截断高频弱振幅项可实现平滑效果并缩短输出公式。

### 3.3 模板驱动的多端序列化 (Template-Driven Output)

系统抛弃硬编码字符串拼接，全面采用 `tera` 模板引擎，实现“一次计算，多端输出”。系统必须支持以下序列化生成：

- **F-OUT-01: LaTeX 数学公式 (`.tex`)**
  - 生成标准 LaTeX 语法，支持分段函数 `cases` 环境。输出干净、易读的三角函数求和式。
- **F-OUT-02: Desmos API 集成环境 (`.html`)**
  - 渲染一个内嵌 Desmos Calculator API 的完整 HTML 页面。自动计算并注入 `calculator.setMathBounds()`，并将所有多项式通过 API 注入左侧表达式列表。
- **F-OUT-03: Python (Matplotlib) 可执行脚本 (`.py`)**
  - 自动引入 `numpy` 构建致密的 $t$ 向量，生成 `x = a*np.cos(...)` 风格的向量化计算代码，并通过 `plt.plot()` 绘制闭合曲线。
- **F-OUT-04: MATLAB / GNU Octave 脚本 (`.m`)**
  - 充分利用 MATLAB 的矩阵运算特性，生成 `.m` 脚本，无需 for 循环即可极速渲染极长项数的傅里叶方程组。
- **F-OUT-05: Julia (Plots.jl) 脚本 (`.jl`)**
  - 利用 Julia 语言在科学计算上的优势，生成高度优化的宏式代码或原生的函数绘图调用。
- **F-OUT-06: GeoGebra 指令批处理 (`.ggb.txt`)**
  - 将复杂的方程组拆解转化为 GeoGebra 控制台可直接识别的 `Curve[x(t), y(t), t, t_min, t_max]` 原生指令流。
- **F-OUT-07: Raw JSON 抽象语法树 (`.json`)**
  - 针对高级二次开发者，提供纯数据格式输出，包含所有计算完毕的频率、振幅、相位系数及节点坐标数组，便于集成至其他软件流水线。

## 4. 非功能性需求 (Non-Functional Requirements)

### 4.1 性能指标与资源消耗 (Performance)

- **执行时间**: 对于中等分辨率（1080x1080）的光栅图，包含上万个边缘像素点的 TSP 路径规划与 FFT 变换总耗时必须控制在 **3 秒以内**（在主流四核 CPU 上）。
- **内存占用**: 具备流式处理意识，程序运行时的峰值内存占用 (Peak RAM) 应严格限制在 **500MB 以下**，严禁因点云过大导致的 OOM (Out Of Memory)。
- **多线程加速**: 针对 RDP 降采样和 2-Opt 路径优化等计算密集型任务，需无缝集成 `rayon` 库进行数据并行处理。

### 4.2 容错性与异常监控 (Robustness)

- **安全崩溃防御**: 面对破损的 SVG 文件、无法识别的图像编码或全白无轮廓的废弃输入，系统需使用 Rust 的 `Result/Option` 机制逐层向上抛出错误，严禁直接触发 `panic!`。
- **友好的 Terminal 反馈**: 异常退出时，必须打印带有颜色高亮的明确提示（如 `[ERROR] No closed contour found in the image. Try adjusting the threshold parameter.`）。
- **数学奇点保护**: 在生成一次函数（直线）的公式时，若检测到垂直直线（斜率分母趋近于0），必须自动转换方程形式为 $x = C$，避免出现除以零异常。

### 4.3 易用性与交互体验 (Usability)

- **CLI 进度反馈**: 鉴于 TSP 计算可能存在感知延迟，命令行必须使用 `indicatif` 库展示带有平滑动画的 Progress Bar（进度条）与预估剩余时间（ETA）。

- **参数设计直观**: CLI 设计应符合 POSIX 规范。提供完善的 `--help` 帮助文档，例如：

  `vectomancy run input.png --mode fourier --terms 1000 --format python --output output.py --verbose`

- **日志系统**: 提供 `--verbose` 或 `-v` 参数，通过 `env_logger` 输出底层的调试信息（如解析到的 SVG 节点数、TSP 降采样前后的点数对比等）。

### 4.4 可维护性与代码架构 (Maintainability)

- **模块化**: 必须将核心算法（Math/FFT）、解析器（Parsers）和输出生成器（Emitters）拆分为 Rust 的不同子模块 (Modules) 甚至内部 Crate。
- **单元测试**: 对 FFT 频率还原的准确性、RDP 降采样的预期行为、SVG 坐标矩阵变换等核心算子，测试覆盖率 (Coverage) 必须大于 85%。

## 5. 系统架构设计 (Architecture & Data Flow)

### 5.1 核心层级划分

系统整体分为四个逻辑层级，通过严格定义的数据结构体 (Structs) 在层级间传递数据：

1. **CLI 交互层 (Interface Layer)**: 使用 `clap` crate 解析用户参数，初始化日志记录器，并调度底层任务。
2. **多态解析层 (Parsing Layer)**:
   - `image` crate 负责将像素网格提纯为 `Vec<Point2D>`。
   - `usvg` crate 负责将 XML 树转换为 `Vec<BezierSegment>`。
3. **计算调度层 (Engine Layer)**:
   - 包含 TSP 调度器、FFT 计算核心 (基于 `rustfft`)。
   - 在此层，所有多态图形数据被统一抽象为一组通用的内部数据结构 `MathExpressionAST`（数学表达式抽象语法树）。
4. **渲染发射层 (Emitter Layer)**:
   - 搭载 `tera` 模板引擎。将 `MathExpressionAST` 注入预置在二进制文件中的 `.tera` 模板文件，执行渲染并实施磁盘 I/O。

### 5.2 数据流向图解 (Data Flow)

```
[Input.png] -> (Grayscale & Canny) -> [Raw Point Cloud] -> (RDP) -> [Reduced Points]
                                                                        |
                                                                  (TSP 2-Opt)
                                                                        |
[Ordered Point Signal z(t)] -> (Fast Fourier Transform) -> [Freq, Amplitude, Phase]
                                                                        |
                                                         (Construct MathExpressionAST)
                                                                        |
                                                            [Tera Template Engine]
                                                                  /     |      \
                                                        [output.py] [out.tex] [out.html]
```

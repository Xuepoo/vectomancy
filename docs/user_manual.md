# Vectomancy 用户使用手册 (User Manual)

欢迎使用 **Vectomancy**！Vectomancy 是一个高性能的命令行图像转换工具。它能深入解析光栅图像和矢量文件，将它们转化为极具数学美感的参数方程集合，并且直接渲染为各大数学软件支持的工程格式或脚本。

## 1. 核心功能

- **多格式数学公式导出**：支持 Python (Matplotlib), LaTeX (TikZ), Wolfram, GeoGebra (`.ggb`), Kmplot (`.fkt`), HTML5 Canvas, 以及原生 JSON。
- **AST 体积优化**：使用 `Zlib + Base64` 编码来存储巨大的浮点数矩阵，不仅使生成文件小巧，更避免了编辑器和渲染引擎在打开文件时发生的解析树(AST)卡死崩溃问题。
- **可控的光滑度与渲染模式**：
  - `--mode spline`：以精确的贝塞尔曲线插值还原形状，搭配 Chaikin 算法平滑处理，解决阶梯锯齿感。
  - `--mode fourier`：利用傅里叶级数（基于 TSP 旅行商路径规划），逼近生成一条连续的一笔画图像曲线。
- **轻量化与容差配置**：针对 GeoGebra/Kmplot 等纯公式解析软件，提供了 `--tolerance`（RDP 算法容差）与 `--min-path-len` 参数，以便在不过度失真的情况下大幅过滤噪点，控制生成方程的数量，有效避免渲染卡顿。

## 2. 快速开始

### 2.1 安装方式

我们提供了各个平台的构建版本：

- **下载预编译二进制**：你可以在本仓库的 [Release](https://github.com/Xuepoo/vectomanct/releases) 页面下载对应于 Windows, macOS, Linux (Debian, Arch, RedHat, openSUSE, NixOS 等) 平台的原生二进制文件。
- **从源码编译 (Rust)**：
  ```bash
  git clone https://github.com/Xuepoo/vectomanct.git
  cd vectomanct/vectomancy
  cargo build --release
  ```

### 2.2 基础用法

如果你有一张图片（例如 `assets/Tux.png`），你可以通过以下命令将其转换为 Python 脚本：

```bash
./vectomancy run assets/Tux.png --output Tux.py --format python --mode spline
```

如果你希望在数学软件（比如 GeoGebra）中双击直接打开渲染出的方程图像，可以运行：

```bash
./vectomancy run assets/Tux.png --output Tux.ggb --format geogebra --mode spline --chaikin-iters 2 --tolerance 2.0
```

_(注意：对于 GeoGebra，建议调高 `--tolerance` 如 2.0，以大幅降低总方程数，从而保证软件流畅运行。)_

## 3. 高级配置参数

你可以使用 `vectomancy --help` 查看所有的参数。其中常用参数如下：

- `--format <FORMAT>`：输出文件格式。可选：`python`, `latex`, `html`, `json`, `geogebra`, `wolfram`, `kmplot`。
- `--mode <MODE>`：计算模式。可选：`spline`（贝塞尔曲线方程，推荐用于精确显示）, `fourier`（傅里叶级数一笔画逼近）。
- `--chaikin-iters <N>`：针对 Spline 模式时进行的 Chaikin 迭代平滑次数，默认为 `0`。数值越高，生成的折线转角越圆滑。推荐设置 `1` 或 `2`。
- `--tolerance <FLOAT>`：Ramer-Douglas-Peucker 算法简化容差。数值越大，省略的顶点和方程组就越多。渲染大图至数学软件时，推荐 `2.0`。
- `--min-path-len <FLOAT>`：忽略的总长度低于该值的噪点路径。提高该值可去除图像转换过程中的细碎斑点。

## 4. 配置文件 (Configuration File)

用户可以通过在系统配置目录中创建 `config.toml` 来配置默认设置。
Linux 系统路径为 `~/.config/vectomancy/config.toml`。

示例 `config.toml`:

```toml
chaikin_iters = 2
tolerance = 1.5
min_path_len = 5.0
```

## 5. 前置要求与输出使用 (Prerequisites and Output Usage)

- **Python**: 需要 `python3` 和 `matplotlib` (`pip install matplotlib`)。运行命令：`python3 output.py`。
- **LaTeX**: 需要 `texlive-latexextra` 或支持 TikZ 的 TeX 发行版。编译命令：`pdflatex output.tex`。
- **Wolfram**: 需要 Wolfram Engine (`wolframscript`)。运行命令：`wolframscript -f output.txt`。
- **GeoGebra**: 直接在 GeoGebra 应用中打开生成的 `.ggb` (ZIP 压缩包)。
- **Kmplot**: 直接在 Kmplot 中打开生成的 `.fkt` XML 文件。
- **HTML**: 直接在现代浏览器（Chrome, Firefox）中打开。

## 6. 使用示例 (Examples)

- **高质量 Python 样条曲线生成**:
  `./vectomancy run input.png --output out.py --format python --mode spline --chaikin-iters 2`
- **低密度数学软件渲染**:
  `./vectomancy run input.png --output out.ggb --format geogebra --mode spline --tolerance 2.0`

## 7. 常见问题排查

**Q: 打开生成的 Python 或 HTML 文件，我的 VSCode 会卡死吗？**
**A:** 不会。自 1.0 版本起，我们自动在生成的脚本开头注入了防扫描指令（如 `# pylint: disable=all` 或 `<!-- eslint-disable -->`）。且通过 Zlib 压缩，文件体积在几百 KB 级别，主流 IDE 都可以安全打开。

**Q: 为什么我导入 GeoGebra 会卡死？**
**A:** GeoGebra 等数学公式渲染软件受限于内部 XML 树解析限制，如果图片包含过多噪点导致方程多达上万条，就会卡顿。建议通过增加 `--tolerance`（比如设定为 2.0 或 3.0）并指定 `--min-path-len` 以滤除细碎线条。

---

感谢使用 Vectomancy，尽情享受数学曲线所带来的视觉艺术吧！

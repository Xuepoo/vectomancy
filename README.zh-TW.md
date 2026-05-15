# Vectomancy

[English](README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [Français](README.fr.md) | [Español](README.es.md)

Vectomancy 是一個高效能的命令列介面工具，專為解析圖形檔案並將其轉換為數學參數方程式及渲染腳本而設計。它讓使用者能夠將點陣圖像和向量圖形轉化為數學上美麗的波形。

## 範例展示

| 原始圖像                                                      | 渲染輸出 (無顏色)                                                     | 渲染輸出 (彩色)                                                             |
| :------------------------------------------------------------ | :-------------------------------------------------------------------- | :-------------------------------------------------------------------------- |
| ![Original Image](https://cdn.xuepoo.xyz/dolphin.jpg)         | ![Rendered Output](https://cdn.xuepoo.xyz/dolphin_render.png)         | ![Rendered Output](https://cdn.xuepoo.xyz/dolphin_render_color.png)         |
| ![Original Image](https://cdn.xuepoo.xyz/Hatsune_miku_v2.png) | ![Rendered Output](https://cdn.xuepoo.xyz/Hatsune_miku_v2_render.png) | ![Rendered Output](https://cdn.xuepoo.xyz/Hatsune_miku_v2_render_color.png) |
| ![Original Image](https://cdn.xuepoo.xyz/Tux.png)             | ![Rendered Output](https://cdn.xuepoo.xyz/Tux_render.png)             | ![Rendered Output](https://cdn.xuepoo.xyz/Tux_render_color.png)             |
| ![Original Image](https://cdn.xuepoo.xyz/01_khafre_north.jpg) | ![Rendered Output](https://cdn.xuepoo.xyz/01_khafre_north_render.png) | ![Rendered Output](https://cdn.xuepoo.xyz/01_khafre_north_render_color.png) |

### 圖像來源

- Dolphin: [https://en.wikipedia.org/wiki/Guiana_dolphin#/media/File:Descri%C3%A7%C3%A3o_in%C3%ADcio_ou_comportamento.jpg](https://en.wikipedia.org/wiki/Guiana_dolphin#/media/File:Descri%C3%A7%C3%A3o_in%C3%ADcio_ou_comportamento.jpg)
- Miku: [https://storage.moegirl.org.cn/moegirl/commons/3/35/Hatsune_miku_v2.png](https://storage.moegirl.org.cn/moegirl/commons/3/35/Hatsune_miku_v2.png)
- Tux: [https://en.wikipedia.org/wiki/File:Tux.svg](https://en.wikipedia.org/wiki/File:Tux.svg)
- Pyramid: [https://en.wikipedia.org/wiki/Pyramid#/media/File:01_khafre_north.jpg](https://en.wikipedia.org/wiki/Pyramid#/media/File:01_khafre_north.jpg)

## 核心功能

- **多格式數學公式匯出**：支援 Python (Matplotlib), HTML5 Canvas，以及原生 JSON。
- **AST 體積最佳化**：使用 `Zlib + Base64` 編碼來儲存龐大的浮點數矩陣，這不僅使生成的檔案保持小巧，也避免了編輯器和渲染引擎在解析大檔案時發生凍結或崩潰。
- **可控的平滑度與渲染模式**：
  - `--mode spline`：以精確的貝茲曲線插值重建形狀，並結合 Chaikin 演算法進行平滑處理，消除鋸齒狀的階梯邊緣。
  - `--mode fourier`：利用傅立葉級數（基於 TSP 路徑規劃）近似生成一條連續的一筆畫圖像曲線。

如需深入了解底層的數學演算法（例如 Otsu 二值化、Ramer-Douglas-Peucker 降維、Moore 鄰域追蹤和 FFT），請參閱[使用者手冊](docs/user_manual.md)。

## 安裝方式

從原始碼編譯需要先安裝 Rust 工具鏈：

```bash
git clone https://github.com/Xuepoo/vectomancy.git
cd vectomancy/vectomancy
cargo build --release
```

您也可以在 [GitHub Releases](https://github.com/Xuepoo/vectomancy/releases) 頁面下載適用於 Linux (Debian, Arch, RedHat, openSUSE, NixOS), Windows 和 macOS 平台的預先編譯原生二進位檔案。

## CLI 基礎用法

```bash
vectomancy run [OPTIONS] --output <OUTPUT> <INPUT>
```

選項:

- `-o, --output <OUTPUT>`: 生成檔案的輸出路徑。
- `-f, --format <FORMAT>`: 輸出格式 (python, html, json)。
- `-m, --mode <MODE>`: 轉換模式 (fourier, spline)。
- `-n, --terms <TERMS>`: 傅立葉級數近似項數 (預設: 1000)。

設定檔會遵循 XDG 規範從 `~/.config/vectomancy/config.toml` 中載入。

## 常見問題 (FAQ)

**Q: 打開生成的 Python 或 HTML 檔案時 VSCode 會凍結嗎？**
**A:** 不會。我們會在生成的腳本開頭自動插入反掃描指令（如 `# pylint: disable=all` 或 `<!-- eslint-disable -->`）。透過 Zlib 壓縮，檔案大小保持得很小，主流 IDE 皆可安全開啟。



## 授權條款

本專案採用 MIT 授權條款。

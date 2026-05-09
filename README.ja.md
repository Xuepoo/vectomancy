# Vectomancy

Vectomancy は、さまざまなグラフィックファイルを解析し、数学的なパラメータ方程式やレンダリングスクリプトに変換するように設計された高性能なコマンドラインインターフェースツールです。ユーザーは、ラスター画像やベクターグラフィックスを数学的な波形に変換できます。

## 機能

- **入力解析と前処理:**
  - **ベクター (`.svg`):** パス、変換、基本形状を正規化された絶対座標に解析します。
  - **ラスター (`.png`, `.jpg`, `.webp`):** Ramer-Douglas-Peucker (RDP) アルゴリズムを使用して、ノイズ除去、二値化、輪郭追跡、スケルトン化、点群削減を行います。
- **数学変換エンジン:**
  - **フーリエ級数近似 (`--mode fourier`):** TSP (最近傍法 + 2-Opt) を使用して最適な連続パスを見つけ、FFT を適用して設定可能な項数 (`--terms`) でパスを近似します。ラスター入力や複雑な非パラメータ形状に最適です。
  - **正確なパラメータスプライン (`--mode spline`):** SVG ベジェ曲線を正確なパラメータ多項式方程式グループに変換します。
- **テンプレート駆動型出力:** LaTeX (`.tex`)、Desmos HTML (`.html`)、Python Matplotlib (`.py`)、GeoGebra コマンド (`.ggb.txt`)、生の JSON (`.json`) など、さまざまな形式で出力を生成します。

## コアアルゴリズム

エンジンは、正確な変換を実現するためにいくつかの技術を採用しています。

- **大津の二値化 (Otsu Binarization)**: 画像二値化の最適な閾値を自動的に決定します。
- **ムーア近傍追跡 (Moore Neighborhood Tracing)**: 二値画像から輪郭を抽出します。
- **Ramer-Douglas-Peucker 削減**: 形状を維持しながら点数を減らしてパスを簡略化します。
- **TSP 最近傍法 + 2-Opt**: フーリエ級数近似のためにパスの連続性を最適化します。
- **FFT (高速フーリエ変換)**: 設定可能な項数を使用してパスを近似します。

## 示例展示

| オリジナル画像                                     | レンダリング出力                                            |
| :------------------------------------------------- | :---------------------------------------------------------- |
| ![オリジナル画像](assets/Hatsune_miku_v2.png)      | ![レンダリング出力](assets/Hatsune_miku_v2_render.png)      |
| ![オリジナル画像](assets/Tux.svg)                  | ![レンダリング出力](assets/Tux_render.png)                  |
| ![オリジナル画像](assets/Cat_November_2010-1a.jpg) | ![レンダリング出力](assets/Cat_November_2010-1a_render.png) |
| ![オリジナル画像](assets/01_khafre_north.jpg)      | ![レンダリング出力](assets/01_khafre_north_render.png)      |

### 画像ソース

- Miku: [https://storage.moegirl.org.cn/moegirl/commons/3/35/Hatsune_miku_v2.png](https://storage.moegirl.org.cn/moegirl/commons/3/35/Hatsune_miku_v2.png)
- Tux: [https://en.wikipedia.org/wiki/File:Tux.svg](https://en.wikipedia.org/wiki/File:Tux.svg)
- Cat: [https://en.wikipedia.org/wiki/Tabby_cat#/media/File:Cat_November_2010-1a.jpg](https://en.wikipedia.org/wiki/Tabby_cat#/media/File:Cat_November_2010-1a.jpg)
- Pyramid: [https://en.wikipedia.org/wiki/Pyramid#/media/File:01_khafre_north.jpg](https://en.wikipedia.org/wiki/Pyramid#/media/File:01_khafre_north.jpg)

## CLI の使用方法

```bash
vectomancy run [OPTIONS] --output <OUTPUT> <INPUT>
```

オプション:

- `-o, --output <OUTPUT>`: 生成された出力ファイルのパス。
- `-f, --format <FORMAT>`: 出力形式 (python, latex, html, json, geogebra, wolfram)。
- `-m, --mode <MODE>`: 変換モード (fourier, spline)。
- `-n, --terms <TERMS>`: フーリエ近似の項数 (デフォルト: 1000)。

設定は、XDG Base Directory 仕様に従い `~/.config/vectomancy/config.toml` から読み込まれます。

## ロードマップ

- Compute Shader (wgpu および Vulkan) による GPU アクセラレーション。
- マルチスレッドの改善。
- カラーターミナル出力。

## ライセンス

このプロジェクトは MIT ライセンスの下でライセンスされています。

## インストール

Rust ツールチェーンをインストールする必要があります。

```bash
cargo build --release
```

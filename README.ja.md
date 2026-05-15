# Vectomancy

[English](README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [Français](README.fr.md) | [Español](README.es.md)

Vectomancy は、画像ファイルを解析し、数学的なパラメータ方程式とレンダリングスクリプトに変換するために設計された、高性能なコマンドラインインターフェースツールです。ラスター画像やベクターグラフィックスを数学的に美しい波形に変換することができます。

## ギャラリー

| 元の画像                                                      | レンダリング出力 (色なし)                                             | レンダリング出力 (色あり)                                                   |
| :------------------------------------------------------------ | :-------------------------------------------------------------------- | :-------------------------------------------------------------------------- |
| ![Original Image](https://cdn.xuepoo.xyz/dolphin.jpg)         | ![Rendered Output](https://cdn.xuepoo.xyz/dolphin_render.png)         | ![Rendered Output](https://cdn.xuepoo.xyz/dolphin_render_color.png)         |
| ![Original Image](https://cdn.xuepoo.xyz/Hatsune_miku_v2.png) | ![Rendered Output](https://cdn.xuepoo.xyz/Hatsune_miku_v2_render.png) | ![Rendered Output](https://cdn.xuepoo.xyz/Hatsune_miku_v2_render_color.png) |
| ![Original Image](https://cdn.xuepoo.xyz/Tux.png)             | ![Rendered Output](https://cdn.xuepoo.xyz/Tux_render.png)             | ![Rendered Output](https://cdn.xuepoo.xyz/Tux_render_color.png)             |
| ![Original Image](https://cdn.xuepoo.xyz/01_khafre_north.jpg) | ![Rendered Output](https://cdn.xuepoo.xyz/01_khafre_north_render.png) | ![Rendered Output](https://cdn.xuepoo.xyz/01_khafre_north_render_color.png) |

### 画像の出典

- Dolphin: [https://en.wikipedia.org/wiki/Guiana_dolphin#/media/File:Descri%C3%A7%C3%A3o_in%C3%ADcio_ou_comportamento.jpg](https://en.wikipedia.org/wiki/Guiana_dolphin#/media/File:Descri%C3%A7%C3%A3o_in%C3%ADcio_ou_comportamento.jpg)
- Miku: [https://storage.moegirl.org.cn/moegirl/commons/3/35/Hatsune_miku_v2.png](https://storage.moegirl.org.cn/moegirl/commons/3/35/Hatsune_miku_v2.png)
- Tux: [https://en.wikipedia.org/wiki/File:Tux.svg](https://en.wikipedia.org/wiki/File:Tux.svg)
- Pyramid: [https://en.wikipedia.org/wiki/Pyramid#/media/File:01_khafre_north.jpg](https://en.wikipedia.org/wiki/Pyramid#/media/File:01_khafre_north.jpg)

## 主な機能

- **マルチフォーマットでの数式エクスポート**: Python (Matplotlib)、HTML5 Canvas、およびネイティブ JSON をサポートしています。
- **AST サイズの最適化**: 膨大な浮点数マトリックスを保存するために `Zlib + Base64` エンコーディングを使用しています。これにより、生成されたファイルがコンパクトに保たれ、大きなファイルをパースする際のエディターやレンダリングエンジンのフリーズやクラッシュを防ぎます。
- **制御可能な滑らかさとレンダリングモード**:
  - `--mode spline`: 正確なベジェ曲線補間で形状を再構築し、Chaikinアルゴリズムと組み合わせてギザギザの階段状のエッジを排除する平滑化を行います。
  - `--mode fourier`: フーリエ級数（TSP経路計画に基づく）を利用して、画像の一筆書きの連続曲線に近似させます。

大津の二値化、Ramer-Douglas-Peucker 削減、Moore近傍追跡、FFTなどの数学的アルゴリズムの詳細については、[ユーザーマニュアル](docs/user_manual.md)を参照してください。

## インストール

ソースからビルドするには、Rustツールチェーンがインストールされている必要があります。

```bash
git clone https://github.com/Xuepoo/vectomancy.git
cd vectomancy/vectomancy
cargo build --release
```

Linux (Debian、Arch、RedHat、openSUSE、NixOS)、Windows、macOS 用のコンパイル済みバイナリは、[GitHub Releases](https://github.com/Xuepoo/vectomancy/releases) で入手できます。

## CLI の使用方法

```bash
vectomancy run [OPTIONS] --output <OUTPUT> <INPUT>
```

オプション:

- `-o, --output <OUTPUT>`: 生成される出力ファイルのパス。
- `-f, --format <FORMAT>`: 出力フォーマット (python, html, json)。
- `-m, --mode <MODE>`: 変換モード (fourier, spline)。
- `-n, --terms <TERMS>`: フーリエ近似の項数 (デフォルト: 1000)。

設定は、XDG Base Directory 仕様に従って `~/.config/vectomancy/config.toml` から読み込まれます。

## FAQ

**Q: 生成された Python や HTML ファイルを開くと VSCode がフリーズしますか？**
**A:** いいえ。生成されたスクリプトの先頭にアンチスキャンディレクティブ（`# pylint: disable=all` や `<!-- eslint-disable -->` など）を自動的に挿入します。Zlib 圧縮によりファイルサイズは小さく保たれ、主要な IDE で安全に開くことができます。



## ライセンス

このプロジェクトは MIT ライセンスの下でライセンスされています。

# Vectomancy

[English](README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [Français](README.fr.md) | [Español](README.es.md)

Vectomancy es una herramienta de interfaz de línea de comandos de alto rendimiento diseñada para analizar archivos gráficos y convertirlos en ecuaciones paramétricas matemáticas y scripts de renderizado. Permite a los usuarios transformar imágenes rasterizadas y gráficos vectoriales en formas de onda matemáticamente hermosas.

## Ejemplos

| Imagen Original                                               | Salida Renderizada (Sin Color)                                        | Salida Renderizada (Con Color)                                              |
| :------------------------------------------------------------ | :-------------------------------------------------------------------- | :-------------------------------------------------------------------------- |
| ![Original Image](https://cdn.xuepoo.xyz/dolphin.jpg)         | ![Rendered Output](https://cdn.xuepoo.xyz/dolphin_render.png)         | ![Rendered Output](https://cdn.xuepoo.xyz/dolphin_render_color.png)         |
| ![Original Image](https://cdn.xuepoo.xyz/Hatsune_miku_v2.png) | ![Rendered Output](https://cdn.xuepoo.xyz/Hatsune_miku_v2_render.png) | ![Rendered Output](https://cdn.xuepoo.xyz/Hatsune_miku_v2_render_color.png) |
| ![Original Image](https://cdn.xuepoo.xyz/Tux.png)             | ![Rendered Output](https://cdn.xuepoo.xyz/Tux_render.png)             | ![Rendered Output](https://cdn.xuepoo.xyz/Tux_render_color.png)             |
| ![Original Image](https://cdn.xuepoo.xyz/01_khafre_north.jpg) | ![Rendered Output](https://cdn.xuepoo.xyz/01_khafre_north_render.png) | ![Rendered Output](https://cdn.xuepoo.xyz/01_khafre_north_render_color.png) |

### Fuentes de las imágenes

- Dolphin: [https://en.wikipedia.org/wiki/Guiana_dolphin#/media/File:Descri%C3%A7%C3%A3o_in%C3%ADcio_ou_comportamento.jpg](https://en.wikipedia.org/wiki/Guiana_dolphin#/media/File:Descri%C3%A7%C3%A3o_in%C3%ADcio_ou_comportamento.jpg)
- Miku: [https://storage.moegirl.org.cn/moegirl/commons/3/35/Hatsune_miku_v2.png](https://storage.moegirl.org.cn/moegirl/commons/3/35/Hatsune_miku_v2.png)
- Tux: [https://en.wikipedia.org/wiki/File:Tux.svg](https://en.wikipedia.org/wiki/File:Tux.svg)
- Pyramid: [https://en.wikipedia.org/wiki/Pyramid#/media/File:01_khafre_north.jpg](https://en.wikipedia.org/wiki/Pyramid#/media/File:01_khafre_north.jpg)

## Características

- **Exportación de ecuaciones matemáticas en múltiples formatos**: Soporta Python (Matplotlib), LaTeX (TikZ), Wolfram, GeoGebra (`.ggb`), Kmplot (`.fkt`), HTML5 Canvas y JSON nativo.
- **Optimización del tamaño del AST**: Utiliza codificación `Zlib + Base64` para almacenar matrices de punto flotante masivas. Esto mantiene los archivos generados compactos y evita que los editores y los motores de renderizado se congelen o se bloqueen al analizar archivos grandes.
- **Modos de renderizado y suavizado controlables**:
  - `--mode spline`: Reconstruye formas con interpolación precisa de curvas Bézier, combinada con el algoritmo de Chaikin para suavizar y eliminar bordes dentados en forma de escalera.
  - `--mode fourier`: Utiliza series de Fourier (basadas en la planificación de rutas TSP) para aproximar una curva continua de un solo trazo de la imagen.

Para profundizar en los detalles de los algoritmos matemáticos (como la binarización de Otsu, la reducción de Ramer-Douglas-Peucker, el trazado de vecindad de Moore y la FFT), consulte el [Manual del usuario](docs/user_manual.md).

## Instalación

Necesitará tener instalada la cadena de herramientas de Rust para compilar desde el código fuente.

```bash
git clone https://github.com/Xuepoo/vectomancy.git
cd vectomancy/vectomancy
cargo build --release
```

Los binarios precompilados para Linux (Debian, Arch, RedHat, openSUSE, NixOS), Windows y macOS están disponibles en [GitHub Releases](https://github.com/Xuepoo/vectomancy/releases).

## Uso de la CLI

```bash
vectomancy run [OPTIONS] --output <OUTPUT> <INPUT>
```

Opciones:

- `-o, --output <OUTPUT>`: Ruta para el archivo de salida generado.
- `-f, --format <FORMAT>`: Formato de salida (python, latex, html, json, geogebra, wolfram, kmplot).
- `-m, --mode <MODE>`: Modo de conversión (fourier, spline).
- `-n, --terms <TERMS>`: Número de términos para la aproximación de Fourier (predeterminado: 1000).

La configuración se carga desde `~/.config/vectomancy/config.toml` siguiendo la especificación XDG Base Directory.

## Preguntas frecuentes (FAQ)

**Q: ¿Se congelará mi VSCode al abrir los archivos Python o HTML generados?**
**A:** No. Inyectamos automáticamente directivas antiescaneo (como `# pylint: disable=all` o `<!-- eslint-disable -->`) al comienzo de los scripts generados. A través de la compresión Zlib, los tamaños de los archivos se mantienen pequeños, lo que los principales IDE pueden abrir de forma segura.

**Q: ¿Por qué GeoGebra se congela cuando importo el archivo?**
**A:** El software de renderizado de fórmulas matemáticas está limitado por restricciones internas de análisis de árboles XML. Si una imagen contiene demasiado ruido que da como resultado decenas de miles de ecuaciones, se ralentizará. Recomendamos aumentar la tolerancia `--tolerance` (por ejemplo, a 2.0 o 3.0) y especificar `--min-path-len` para filtrar pequeñas líneas ruidosas. Consulte el [Manual del usuario](docs/user_manual.md) para opciones de ajuste detalladas.

## Licencia

Este proyecto está licenciado bajo la Licencia MIT.

# Vectomancy

[English](README.md) | [简体中文](README.zh-CN.md) | [繁體中文](README.zh-TW.md) | [日本語](README.ja.md) | [Français](README.fr.md) | [Español](README.es.md)

Vectomancy est un outil d'interface en ligne de commande très performant conçu pour analyser des fichiers graphiques et les convertir en équations paramétriques mathématiques et en scripts de rendu. Il permet aux utilisateurs de transformer des images matricielles et des graphiques vectoriels en formes d'ondes mathématiquement magnifiques.

## Exemples de rendus

| Image Originale                               | Sortie Rendue                                         |
| :-------------------------------------------- | :---------------------------------------------------- |
| ![Original Image](assets/dolphin.jpg)         | ![Rendered Output](assets/dolphin_render.png)         |
| ![Original Image](assets/Hatsune_miku_v2.png) | ![Rendered Output](assets/Hatsune_miku_v2_render.png) |
| ![Original Image](assets/Tux.png)             | ![Rendered Output](assets/Tux_render.png)             |
| ![Original Image](assets/01_khafre_north.jpg) | ![Rendered Output](assets/01_khafre_north_render.png) |

### Sources des images

- Dolphin: [https://en.wikipedia.org/wiki/Guiana_dolphin#/media/File:Descri%C3%A7%C3%A3o_in%C3%ADcio_ou_comportamento.jpg](https://en.wikipedia.org/wiki/Guiana_dolphin#/media/File:Descri%C3%A7%C3%A3o_in%C3%ADcio_ou_comportamento.jpg)
- Miku: [https://storage.moegirl.org.cn/moegirl/commons/3/35/Hatsune_miku_v2.png](https://storage.moegirl.org.cn/moegirl/commons/3/35/Hatsune_miku_v2.png)
- Tux: [https://en.wikipedia.org/wiki/File:Tux.svg](https://en.wikipedia.org/wiki/File:Tux.svg)
- Pyramid: [https://en.wikipedia.org/wiki/Pyramid#/media/File:01_khafre_north.jpg](https://en.wikipedia.org/wiki/Pyramid#/media/File:01_khafre_north.jpg)

## Fonctionnalités

- **Exportation d'équations mathématiques multi-formats** : Supporte Python (Matplotlib), LaTeX (TikZ), Wolfram, GeoGebra (`.ggb`), Kmplot (`.fkt`), HTML5 Canvas et JSON natif.
- **Optimisation de la taille de l'AST** : Utilise l'encodage `Zlib + Base64` pour stocker des matrices massives de points flottants. Cela garde les fichiers générés compacts et empêche les éditeurs et les moteurs de rendu de planter lors de l'analyse de fichiers volumineux.
- **Mode de lissage et de rendu contrôlable** :
  - `--mode spline` : Reconstruit les formes avec une interpolation précise par courbe de Bézier, combinée à l'algorithme de Chaikin pour un lissage qui élimine les bords en escalier.
  - `--mode fourier` : Utilise les séries de Fourier (basées sur la planification de trajectoire TSP) pour approximer une courbe continue d'un seul trait.

Pour approfondir les détails des algorithmes mathématiques (tels que la binarisation d'Otsu, la réduction de Ramer-Douglas-Peucker, le traçage de voisinage de Moore et la FFT), veuillez vous référer au [Manuel de l'utilisateur](docs/user_manual.md).

## Installation

Vous devrez avoir installé la chaîne d'outils Rust pour compiler à partir de la source.

```bash
git clone https://github.com/Xuepoo/vectomancy.git
cd vectomancy/vectomancy
cargo build --release
```

Des binaires précompilés pour Linux (Debian, Arch, RedHat, openSUSE, NixOS), Windows et macOS sont disponibles sur la page [GitHub Releases](https://github.com/Xuepoo/vectomancy/releases).

## Utilisation CLI

```bash
vectomancy run [OPTIONS] --output <OUTPUT> <INPUT>
```

Options :

- `-o, --output <OUTPUT>` : Chemin du fichier de sortie généré.
- `-f, --format <FORMAT>` : Format de sortie (python, latex, html, json, geogebra, wolfram, kmplot).
- `-m, --mode <MODE>` : Mode de conversion (fourier, spline).
- `-n, --terms <TERMS>` : Nombre de termes pour l'approximation de Fourier (par défaut : 1000).

La configuration est chargée à partir de `~/.config/vectomancy/config.toml` conformément à la spécification XDG Base Directory.

## FAQ

**Q: Mon VSCode va-t-il geler lors de l'ouverture des fichiers Python ou HTML générés ?**
**A:** Non. Nous injectons automatiquement des directives anti-analyse (comme `# pylint: disable=all` ou `<!-- eslint-disable -->`) au début des scripts générés. Grâce à la compression Zlib, les tailles de fichiers restent petites, ce qui permet aux IDE grand public de les ouvrir en toute sécurité.

**Q: Pourquoi GeoGebra gèle-t-il lors de l'importation du fichier ?**
**A:** Les logiciels de rendu de formules mathématiques sont limités par des restrictions d'analyse d'arbres XML internes. Si une image contient trop de bruit entraînant des dizaines de milliers d'équations, le logiciel ralentira. Nous recommandons d'augmenter la tolérance `--tolerance` (par ex., 2.0 ou 3.0) et de spécifier `--min-path-len` pour filtrer les petites lignes bruyantes. Consultez le [Manuel de l'utilisateur](docs/user_manual.md) pour les options de réglage détaillées.

## Licence

Ce projet est sous licence MIT.

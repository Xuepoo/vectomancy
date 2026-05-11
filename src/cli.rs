use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "vectomancy", version, author, about = "Image-to-Equation Converter", long_about = None)]
pub struct Cli {
    /// Input file paths (.png, .jpg, .svg) or directories
    pub inputs: Vec<PathBuf>,

    /// Output file path or directory
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output format (python, latex, html, json, geogebra)
    #[arg(short, long)]
    pub format: Option<OutputFormat>,

    /// Processing mode
    #[arg(short, long)]
    pub mode: Option<Mode>,

    /// Number of Fourier terms
    #[arg(short = 'n', long, default_value_t = 1000)]
    pub terms: usize,

    /// Number of Chaikin smoothing iterations (applies to Chaikin mode)
    #[arg(short = 'c', long)]
    pub chaikin_iters: Option<usize>,

    /// Tolerance for RDP simplification
    #[arg(long, default_value_t = 0.5)]
    pub tolerance: f64,

    /// Minimum path length to process
    #[arg(long, default_value_t = 5)]
    pub min_path_len: usize,

    /// Enable color sampling and drawing
    #[arg(long, default_value_t = false)]
    pub color: bool,

    /// Transparent background for native image rendering
    #[arg(long, default_value_t = false)]
    pub bg_transparent: bool,

    /// Target width for native image rendering
    #[arg(long)]
    pub width: Option<u32>,

    /// Target height for native image rendering
    #[arg(long)]
    pub height: Option<u32>,

    /// Stroke width for native image rendering
    #[arg(long, default_value_t = 1.0)]
    pub stroke_width: f32,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Bit depth for native rendering (8, 10, 16, 32)
    #[arg(long)]
    pub bit_depth: Option<u8>,

    /// Color space for native rendering (sRGB, DisplayP3, CMYK)
    #[arg(long)]
    pub color_space: Option<String>,

    /// Enable GPU acceleration (wgpu) - Defaults to CPU
    #[arg(long)]
    pub gpu: bool,

    /// Number of threads for CPU multithreading (default: 1)
    #[arg(long)]
    pub threads: Option<usize>,

    /// GPU Power Preference (HighPerformance, LowPower, None)
    #[arg(long)]
    pub gpu_power: Option<String>,
}

#[derive(ValueEnum, Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Python,
    Latex,
    Html,
    Json,
    Geogebra,
    Wolfram,
    Kmplot,
    Png,
    Jpg,
    Webp,
}

#[derive(ValueEnum, Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Fourier,
    Spline,
    Chaikin,
}

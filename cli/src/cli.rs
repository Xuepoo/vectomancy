use clap::Parser;
use std::path::PathBuf;
pub use vectomancy::config::{Mode, OutputFormat};

#[derive(Parser, Debug)]
#[command(name = "vectomancy", version, author, about = "Image-to-Equation Converter", long_about = None)]
pub struct Cli {
    /// Input file paths (.png, .jpg, .svg) or directories
    pub inputs: Vec<PathBuf>,

    /// Path to a custom config file (.toml)
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Output file path or directory
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output format
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

    /// Detail level for paths (1-100), higher = more equations/detail
    #[arg(long)]
    pub detail: Option<u8>,

    /// Tolerance for RDP simplification (overrides detail)
    #[arg(long)]
    pub tolerance: Option<f64>,

    /// Minimum path length to process
    #[arg(long, default_value_t = 5)]
    pub min_path_len: usize,

    /// Enable color sampling and drawing
    #[arg(long, action = clap::ArgAction::Set)]
    pub color: Option<bool>,

    /// Transparent background for native image rendering
    #[arg(long, action = clap::ArgAction::Set)]
    pub bg_transparent: Option<bool>,

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
    #[arg(long, action = clap::ArgAction::Set)]
    pub gpu: Option<bool>,

    /// Number of threads for CPU multithreading (default: 1)
    #[arg(long)]
    pub threads: Option<usize>,

    /// GPU Power Preference (HighPerformance, LowPower, None)
    #[arg(long)]
    pub gpu_power: Option<String>,

    /// Generate shell completions
    #[arg(long)]
    pub generate_completions: Option<clap_complete::Shell>,
}

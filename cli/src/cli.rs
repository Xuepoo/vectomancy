use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
pub use vectomancy::config::{Mode, OutputFormat};

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
#[clap(rename_all = "lowercase")]
pub enum CliOutputFormat {
    Python,
    Html,
    Json,
    Desmos,
    Png,
    Jpg,
    Webp,
}

impl From<CliOutputFormat> for OutputFormat {
    fn from(fmt: CliOutputFormat) -> Self {
        match fmt {
            CliOutputFormat::Python => OutputFormat::Python,
            CliOutputFormat::Html => OutputFormat::Html,
            CliOutputFormat::Json => OutputFormat::Json,
            CliOutputFormat::Desmos => OutputFormat::Desmos,
            CliOutputFormat::Png => OutputFormat::Png,
            CliOutputFormat::Jpg => OutputFormat::Jpg,
            CliOutputFormat::Webp => OutputFormat::Webp,
        }
    }
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
#[clap(rename_all = "lowercase")]
pub enum CliMode {
    Fourier,
    Spline,
    Chaikin,
}

impl From<CliMode> for Mode {
    fn from(mode: CliMode) -> Self {
        match mode {
            CliMode::Fourier => Mode::Fourier,
            CliMode::Spline => Mode::Spline,
            CliMode::Chaikin => Mode::Chaikin,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "vectomancy", version, author, about = "Image-to-Equation Converter", long_about = None)]
pub struct Cli {
    /// Path to a custom config file (.toml)
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Generate shell completions
    #[arg(long, global = true)]
    pub generate_completions: Option<clap_complete::Shell>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Process single raster or vector image
    Image(ImageArgs),

    /// Process video files
    Video(VideoArgs),

    /// Process text input
    Text(TextArgs),
}

#[derive(Args, Debug)]
pub struct ImageArgs {
    /// Input file paths (.png, .jpg, .svg) or directories
    pub inputs: Vec<PathBuf>,

    /// Output file path or directory
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output format
    #[arg(short, long)]
    pub format: Option<CliOutputFormat>,

    /// Processing mode
    #[arg(short, long)]
    pub mode: Option<CliMode>,

    /// Number of Fourier terms
    #[arg(short = 'n', long)]
    pub terms: Option<usize>,

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
    #[arg(long)]
    pub min_path_len: Option<usize>,

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
    #[arg(long)]
    pub stroke_width: Option<f32>,

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

    /// Do not simplify math coordinates and equations (retains original high precision)
    #[arg(long)]
    pub no_simplify_math: bool,
}

#[derive(Args, Debug)]
pub struct VideoArgs {
    /// Input video file path
    pub input: PathBuf,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Do not simplify math coordinates and equations (retains original high precision)
    #[arg(long)]
    pub no_simplify_math: bool,
}

#[derive(Args, Debug)]
pub struct TextArgs {
    /// Input text string
    pub text: String,

    /// Font file path (.ttf or .otf)
    #[arg(short, long)]
    pub font: Option<PathBuf>,

    /// Output file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Transparent background for native image rendering
    #[arg(long, action = clap::ArgAction::Set)]
    pub bg_transparent: Option<bool>,

    /// Solid color in hex format (e.g. #FF0000)
    #[arg(long)]
    pub color: Option<String>,

    /// Gradient in hex format (e.g. #FF0000,#0000FF,45)
    #[arg(long)]
    pub gradient: Option<String>,

    /// Stroke width for native image rendering
    #[arg(long)]
    pub stroke_width: Option<f32>,

    /// Target width for native image rendering
    #[arg(long)]
    pub width: Option<u32>,

    /// Target height for native image rendering
    #[arg(long)]
    pub height: Option<u32>,

    /// Target DPI for native image rendering (default: 72)
    #[arg(long)]
    pub dpi: Option<f32>,

    /// Letter spacing in pixels (can be negative, default: 0.0)
    #[arg(long, default_value_t = 0.0)]
    pub letter_spacing: f32,

    /// Do not simplify math coordinates and equations (retains original high precision)
    #[arg(long)]
    pub no_simplify_math: bool,
}

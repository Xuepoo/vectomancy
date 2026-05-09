use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "vectomancy", version, author, about = "Image-to-Equation Converter", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
    /// Convert an image to mathematical equations
    Run(RunArgs),
}

#[derive(Parser, Debug)]
pub struct RunArgs {
    /// Input file path (.png, .jpg, .svg)
    pub input: PathBuf,

    /// Output file path
    #[arg(short, long)]
    pub output: PathBuf,

    /// Output format (python, latex, html, json, geogebra)
    #[arg(short, long)]
    pub format: Option<OutputFormat>,

    /// Processing mode
    #[arg(short, long)]
    pub mode: Option<Mode>,

    /// Number of Fourier terms
    #[arg(short = 'n', long, default_value_t = 1000)]
    pub terms: usize,

    /// Number of Chaikin smoothing iterations
    #[arg(short = 'c', long)]
    pub chaikin_iters: Option<usize>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
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
}

#[derive(ValueEnum, Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Fourier,
    Spline,
}

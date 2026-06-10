use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    Python,
    Html,
    Json,
    Desmos,
    Png,
    Jpg,
    Webp,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Fourier,
    Spline,
    Chaikin,
}
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct ImageConfig {
    pub mode: Option<Mode>,
    pub terms: Option<usize>,
    pub epsilon: Option<f64>,
    pub format: Option<OutputFormat>,
    pub chaikin_iters: Option<usize>,
    pub detail: Option<u8>,
    pub tolerance: Option<f64>,
    pub min_path_len: Option<usize>,
    pub color: Option<bool>,
    pub default_output_dir: Option<PathBuf>,
    pub bit_depth: Option<u8>,
    pub color_space: Option<String>,
    pub gpu: Option<bool>,
    pub bg_transparent: Option<bool>,
    pub threads: Option<usize>,
    pub gpu_power: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct VideoConfig {
    pub enabled: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct TextConfig {
    pub font: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Config {
    pub image: Option<ImageConfig>,
    pub video: Option<VideoConfig>,
    pub text: Option<TextConfig>,
}

impl Config {
    pub fn load(custom_path: Option<PathBuf>) -> Self {
        let config_file = if let Some(path) = custom_path {
            path
        } else if let Some(proj_dirs) = ProjectDirs::from("", "", "vectomancy") {
            proj_dirs.config_dir().join("config.toml")
        } else {
            return Config::default();
        };

        if config_file.exists() {
            tracing::info!("Loading configuration from {:?}", config_file);
            match fs::read_to_string(&config_file) {
                Ok(contents) => match toml::from_str(&contents) {
                    Ok(config) => return config,
                    Err(e) => {
                        tracing::error!("Failed to parse config file {:?}: {}", config_file, e)
                    }
                },
                Err(e) => tracing::error!("Failed to read config file {:?}: {}", config_file, e),
            }
        } else {
            tracing::debug!("Config file {:?} not found, using defaults", config_file);
        }
        Config::default()
    }
}

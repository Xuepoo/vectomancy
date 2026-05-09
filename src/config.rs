use crate::cli::{Mode, OutputFormat};
use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub mode: Option<Mode>,
    pub terms: Option<usize>,
    pub epsilon: Option<f64>,
    pub format: Option<OutputFormat>,
}

impl Config {
    pub fn load() -> Self {
        if let Some(proj_dirs) = ProjectDirs::from("", "", "vectomancy") {
            let config_dir = proj_dirs.config_dir();
            let config_file = config_dir.join("config.toml");

            if config_file.exists() {
                if let Ok(contents) = fs::read_to_string(&config_file) {
                    if let Ok(config) = toml::from_str(&contents) {
                        return config;
                    } else {
                        tracing::warn!("Failed to parse config.toml");
                    }
                } else {
                    tracing::warn!("Failed to read config.toml");
                }
            }
        }
        Config::default()
    }
}

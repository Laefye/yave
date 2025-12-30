use std::path::Path;

use serde::{Deserialize, Serialize};

pub mod cloudinit;
pub mod vm;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to load configuration: {0}")]
    YAML(#[from] serde_yaml::Error),
    #[error("Configuration file not found at path: {0}")]
    IO(#[from] std::io::Error),
}

type Result<T> = std::result::Result<T, Error>;

fn resolve<P: AsRef<Path>, C: AsRef<Path>>(base: P, relative: C) -> String {
    if relative.as_ref().is_absolute() {
        return relative.as_ref().to_string_lossy().to_string();
    }
    let absolute_base;
    if base.as_ref().is_absolute() == false {
        absolute_base = std::env::current_dir().unwrap().join(base.as_ref());
    } else {
        absolute_base = base.as_ref().to_path_buf();
    }
    let resolved_path = absolute_base.join(relative.as_ref());
    resolved_path.to_string_lossy().to_string()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OVMF {
    pub code: String,
    pub vars: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct API {
    pub groups: Vec<String>,
    pub listen: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub cli: CLI,
    pub ovmf: OVMF,
    pub api: API,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CLI {
    pub bin: String,
    pub img: String,
    pub genisoimage: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let config_str = std::fs::read_to_string(path)?;
        let mut config: Config = serde_yaml::from_str(&config_str)?;
        config.cli.bin = resolve(path, &config.cli.bin);
        config.ovmf.code = resolve(path, &config.ovmf.code);
        config.ovmf.vars = resolve(path, &config.ovmf.vars);
        Ok(config)
    }
}

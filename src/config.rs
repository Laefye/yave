use std::path::Path;

use serde::{Deserialize, Serialize};
use crate::error::{
    ConfigError,
    Error,
};


#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub kvm: KVM,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KVM {
    pub bin: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Error> {
        let config_str = std::fs::read_to_string(path).map_err(ConfigError::from)?;
        let config: Config = toml::from_str(&config_str).map_err(ConfigError::from)?;
        Ok(config)
    }
}

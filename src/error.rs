#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to load configuration: {0}")]
    TOML(#[from] toml::de::Error),
    #[error("Configuration file not found at path: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
}
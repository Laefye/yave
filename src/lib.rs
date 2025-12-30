use crate::{constants::{get_config_path, get_net_script, get_run_path, get_vm_config_path}};

mod constants;
mod tools;
pub mod interface;
pub mod installer;
pub mod contexts;
pub mod vmrunner;
pub(crate) mod db;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO: {0}")]
    IO(#[from] std::io::Error),
    #[error("Config Error: {0}")]
    Config(#[from] vm_types::Error),
    #[error("Serialization Error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("QMP Error: {0}")]
    QMP(#[from] qmp::Error),
    #[error("rtnetlink Error: {0}")]
    Rnetlink(#[from] rtnetlink::Error),
    #[error("Signal Error: {0}")]
    Signal(#[from] nix::Error),
    #[error("Database Error: {0}")]
    Database(#[from] rusqlite::Error),

    // Errors with logic
    #[error("VM Instance is not running: {0}")]
    VMNotRunning(String),
    #[error("VM Instance is already running")]
    VMRunning,
    #[error("VM {0} not found")]
    VMNotFound(String)
}

impl Default for contexts::yave::YaveContext {
    fn default() -> Self {
        Self::load(
            get_config_path(),
            get_vm_config_path(),
            get_run_path(),
            &contexts::yave::NetdevScripts {
                up: get_net_script(true),
                down: get_net_script(false),
            },
        ).expect("Error loading Yave configuration")
    }
}
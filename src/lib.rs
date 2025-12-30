use crate::{constants::{get_config_path, get_net_script, get_run_path, get_vm_config_path}, context::YaveContext};

mod constants;
pub mod interface;
pub mod cloudinit;
pub mod context;
pub mod launch;

pub mod registry;
pub mod storage;
pub mod builders;

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
    Database(#[from] sqlx::Error),

    // Errors with logic
    #[error("VM Instance is not running: {0}")]
    VMNotRunning(String),
    #[error("VM Instance is already running")]
    VMRunning,
    #[error("VM {0} not found")]
    VMNotFound(String)
}

pub struct DefaultYaveContext;

impl DefaultYaveContext {
    pub async fn create() -> Result<YaveContext, crate::Error> {
        let config_path = get_config_path();
        let storage_path = get_vm_config_path();
        let run_path = get_run_path();
        let netdev_scripts = context::NetdevScripts {
            up: get_net_script(true),
            down: get_net_script(false),
        };
        let context = YaveContext::load(config_path, storage_path, run_path, &netdev_scripts).await?;
        Ok(context)
    }
}

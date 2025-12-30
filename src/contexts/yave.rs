use std::path::{Path, PathBuf};

use vm_types::Config;

#[derive(Debug, Clone)]
pub struct YaveContext {
    config: Config,
    storage_path: PathBuf,
    run_path: PathBuf,
    netdev_scripts: NetdevScripts,
}

#[derive(Debug, Clone)]
pub struct NetdevScripts {
    pub up: PathBuf,
    pub down: PathBuf,
}

impl YaveContext {
    pub fn new(config: Config, storage_path: impl AsRef<Path>, run_path: impl AsRef<Path>, netdev_scripts: &NetdevScripts) -> Self
    {
        Self {
            config,
            storage_path: storage_path.as_ref().to_path_buf(),
            run_path: run_path.as_ref().to_path_buf(),
            netdev_scripts: netdev_scripts.clone(),
        }
    }

    pub fn load(config_path: impl AsRef<Path>, storage_path: impl AsRef<Path>, run_path: impl AsRef<Path>, netdev_scripts: &NetdevScripts) -> Result<Self, crate::Error> {
        let config = Config::load(config_path.as_ref())?;
        Ok(Self::new(
            config,
            storage_path,
            run_path,
            netdev_scripts,
        ))
    }

    pub(super) fn storage_path(&self) -> &Path {
        &self.storage_path
    }

    pub(super) fn db_path(&self) -> PathBuf {
        self.storage_path.join("yave.db")
    }

    pub(super) fn run_path(&self) -> &Path {
        &self.run_path
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub(super) fn vm_dir(&self, name: impl ToString) -> PathBuf {
        self.storage_path.join(name.to_string()).with_extension("vm")
    }

    pub(super) fn net_table(&self) -> PathBuf {
        self.storage_path.join("net.table.yaml")
    }

    pub fn netdev_scripts(&self) -> &NetdevScripts {
        &self.netdev_scripts
    }
}

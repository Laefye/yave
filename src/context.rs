use std::path::{Path, PathBuf};

use vm_types::Config;

use crate::{launch::VmRuntime, registry::VmRegistry, storage::VmStorage};

#[derive(Debug, Clone)]
pub struct YaveContext {
    config: Config,
    storage_path: PathBuf,
    run_path: PathBuf,
    netdev_scripts: NetdevScripts,
    db_pool: sqlx::Pool<sqlx::Sqlite>,
}

#[derive(Debug, Clone)]
pub struct NetdevScripts {
    pub up: PathBuf,
    pub down: PathBuf,
}

impl YaveContext {
    pub async fn load(config_path: impl AsRef<Path>, storage_path: impl AsRef<Path>, run_path: impl AsRef<Path>, netdev_scripts: &NetdevScripts) -> Result<Self, crate::Error> {
        let config = Config::load(config_path.as_ref())?;
        if !storage_path.as_ref().exists() {
            std::fs::create_dir_all(storage_path.as_ref())?;
        }
        if !run_path.as_ref().exists() {
            std::fs::create_dir_all(run_path.as_ref())?;
        }
        let db = storage_path.as_ref().join("yave.db");
        if !std::fs::exists(&db)? {
            std::fs::File::create(&db)?;
        }
        Ok(Self {
            config,
            storage_path: storage_path.as_ref().to_path_buf(),
            run_path: run_path.as_ref().to_path_buf(),
            netdev_scripts: netdev_scripts.clone(),
            db_pool: sqlx::SqlitePool::connect(&format!("sqlite://{}", db.to_string_lossy())).await?,
        })
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn netdev_scripts(&self) -> &NetdevScripts {
        &self.netdev_scripts
    }

    pub fn registry(&self) -> VmRegistry {
        VmRegistry::new(self.db_pool.clone())
    }

    pub fn storage(&self) -> VmStorage {
        VmStorage::new(&self.storage_path, &self.config.cli.img)
    }

    pub fn runtime(&self) -> VmRuntime {
        VmRuntime::new(&self.config.cli.bin, &self.run_path, &self.config.ovmf.code, &self.config.ovmf.vars)
    }
}

use std::{path::{Path, PathBuf}, sync::Arc};

use tokio::sync::RwLock;
use vm_types::Config;

use crate::contexts::vm::VirtualMachineContext;

#[derive(Debug, Clone)]
pub struct YaveContext {
    config: Arc<RwLock<Option<Config>>>,
    config_path: PathBuf,
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
    pub fn new(config_path: impl AsRef<Path>, storage_path: impl AsRef<Path>, run_path: impl AsRef<Path>, netdev_scripts: &NetdevScripts) -> Self
    {
        Self {
            config: Arc::new(RwLock::new(None)),
            config_path: config_path.as_ref().to_path_buf(),
            storage_path: storage_path.as_ref().to_path_buf(),
            run_path: run_path.as_ref().to_path_buf(),
            netdev_scripts: netdev_scripts.clone(),
        }
    }

    pub(super) fn storage_path(&self) -> &Path {
        &self.storage_path
    }

    pub(super) fn run_path(&self) -> &Path {
        &self.run_path
    }

    pub async fn config(&self) -> Result<Config, crate::Error> {
        let mut config_lock = self.config.write().await;
        if config_lock.is_none() {
            let config = Config::load(&self.config_path)?;
            *config_lock = Some(config);
        }
        Ok(config_lock.as_ref().unwrap().clone())
    }

    pub(super) fn vm_dir(&self, name: impl ToString) -> PathBuf {
        self.storage_path.join(name.to_string()).with_extension("vm")
    }

    pub(super) fn vnc_table(&self) -> PathBuf {
        self.storage_path.join("vnc.table.yaml")
    }

    pub(super) fn net_table(&self) -> PathBuf {
        self.storage_path.join("net.table.yaml")
    }

    pub fn vm(&self, name: impl ToString) -> VirtualMachineContext {
        VirtualMachineContext::new(self.clone(), self.vm_dir(name).join("config.yaml"))
    }

    fn read_storage(&self) -> impl Iterator<Item = std::fs::DirEntry> {
        std::fs::read_dir(&self.storage_path)
            .into_iter()
            .flatten()
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
    }

    pub fn list_vm(&self) -> Vec<VirtualMachineContext> {
        let mut vms = Vec::new();
        for entry in self.read_storage() {
            let path = entry.path();
            if path.is_dir() && path.extension().map_or(false, |ext| ext == "vm") {
                vms.push(VirtualMachineContext::new(self.clone(), path.join("config.yaml")));
            }
        }
        vms
    }

    pub fn get_vm_by_ifname(&self, ifname: &str) -> Result<Option<VirtualMachineContext>, crate::Error> {
        let tap_table_path = self.net_table();
        let tap_table = vm_types::NetTable::load(&tap_table_path)?;
        if let Some(vm_name) = tap_table.tap.get(ifname) {
            let vm_context = self.vm(&vm_name);
            Ok(Some(vm_context))
        } else {
            Ok(None)
        }
    }

    pub fn netdev_scripts(&self) -> &NetdevScripts {
        &self.netdev_scripts
    }
}

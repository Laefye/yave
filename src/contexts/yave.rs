use std::{cell::RefCell, path::{Path, PathBuf}};

use vm_types::Config;

use crate::contexts::vm::VirtualMachineContext;

#[derive(Debug, Clone)]
pub struct YaveContext {
    config: RefCell<Option<Config>>,
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
            config: RefCell::new(None),
            config_path: config_path.as_ref().to_path_buf(),
            storage_path: storage_path.as_ref().to_path_buf(),
            run_path: run_path.as_ref().to_path_buf(),
            netdev_scripts: netdev_scripts.clone(),
        }
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn storage_path(&self) -> &Path {
        &self.storage_path
    }

    pub fn run_path(&self) -> &Path {
        &self.run_path
    }

    pub fn config(&self) -> Result<Config, crate::Error> {
        if self.config.borrow().is_none() {
            let config = Config::load(&self.config_path)?;
            *self.config.borrow_mut() = Some(config);
        }
        Ok(self.config.borrow().as_ref().unwrap().clone())
    }

    pub(super) fn vm_dir(&self, name: impl ToString) -> PathBuf {
        self.storage_path.join(name.to_string()).with_extension("vm")
    }

    pub fn vnc_table(&self) -> PathBuf {
        self.storage_path.join("vnc.table.yaml")
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

    pub fn list(&self) -> Vec<VirtualMachineContext> {
        let mut vms = Vec::new();
        for entry in self.read_storage() {
            let path = entry.path();
            if path.is_dir() && path.extension().map_or(false, |ext| ext == "vm") {
                vms.push(VirtualMachineContext::new(self.clone(), path.join("config.yaml")));
            }
        }
        vms
    }

    pub fn netdev_scripts(&self) -> &NetdevScripts {
        &self.netdev_scripts
    }
}

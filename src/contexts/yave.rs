use std::path::{Path, PathBuf};

use crate::contexts::vm::VirtualMachineContext;

#[derive(Debug, Clone)]
pub struct YaveContext {
    config_path: PathBuf,
    storage_path: PathBuf,
    run_path: PathBuf,
}

impl YaveContext {
    pub fn new(config_path: impl AsRef<Path>, storage_path: impl AsRef<Path>, run_path: impl AsRef<Path>) -> Self
    {
        Self {
            config_path: config_path.as_ref().to_path_buf(),
            storage_path: storage_path.as_ref().to_path_buf(),
            run_path: run_path.as_ref().to_path_buf(),
        }
    }

    pub fn config(&self) -> &Path {
        &self.config_path
    }

    pub fn storage(&self) -> &Path {
        &self.storage_path
    }

    pub fn run(&self) -> &Path {
        &self.run_path
    }

    fn vm_dir(&self, name: impl ToString) -> PathBuf {
        self.storage_path.join(name.to_string()).with_extension("vm")
    }

    pub fn vnc_table(&self) -> PathBuf {
        self.run_path.join("vnc.table.yaml")
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
}

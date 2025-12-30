use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};
use wincode::{SchemaRead, SchemaWrite};

pub mod utils;
pub mod cloudinit;
pub mod launch;

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

fn unresolve<P: AsRef<Path>, C: AsRef<Path>>(base: P, relative: C) -> String {
    match relative.as_ref().strip_prefix(&base) {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(_) => relative.as_ref().to_string_lossy().to_string(),
    }
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

#[derive(Debug, Deserialize, Serialize, Clone, SchemaWrite, SchemaRead)]
pub struct Hardware {
    pub memory: u32,
    pub vcpu: u32,
    pub ovmf: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone, SchemaWrite, SchemaRead)]
pub struct VNC {
    pub display: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, SchemaWrite, SchemaRead)]
pub struct VirtualMachine {
    pub name: String,
    pub hardware: Hardware,
    pub vnc: VNC,
    pub drives: HashMap<String, Drive>,
    pub networks: HashMap<String, TapInterface>,
}

impl VirtualMachine {
    pub fn unresolve(&self, base: &Path) -> Self {
        let mut unresolved = self.clone();
        for (_, drive) in unresolved.drives.iter_mut() {
            drive.path = unresolve(base, &drive.path);
        }
        unresolved
    }

    pub fn resolve(&self, base: &Path) -> Self {
        let mut resolved = self.clone();
        for (_, drive) in resolved.drives.iter_mut() {
            drive.path = resolve(base, &drive.path);
        }
        resolved
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, SchemaWrite, SchemaRead)]
pub enum MediaType {
    #[serde(rename = "cd")]
    Cdrom,
    #[serde(rename = "hd")]
    Disk,
}

#[derive(Debug, Deserialize, Serialize, Clone, SchemaWrite, SchemaRead)]
pub struct IdeDevice {
    pub media_type: MediaType,
    pub boot_index: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, SchemaWrite, SchemaRead)]
pub struct VirtioBlkDevice {
    pub boot_index: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, SchemaWrite, SchemaRead)]
#[serde(tag = "type")]
pub enum DriveDevice {
    #[serde(rename = "ide")]
    Ide(IdeDevice),
    #[serde(rename = "virtio-blk")]
    VirtioBlk(VirtioBlkDevice)
}

#[derive(Debug, Deserialize, Serialize, Clone, SchemaWrite, SchemaRead)]
pub struct Drive {
    pub path: String,
    pub device: DriveDevice,
}

impl VirtualMachine {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let vm_str = std::fs::read_to_string(path.clone())?;
        let mut vm = serde_yaml::from_str::<VirtualMachine>(&vm_str)?;
        for (_, drive) in vm.drives.iter_mut() {
            drive.path = resolve(&path, &drive.path);
        }
        Ok(vm)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let mut unresolved = self.clone();
        for (_, drive) in unresolved.drives.iter_mut() {
            drive.path = unresolve(&path.as_ref().parent().unwrap_or(Path::new(".")), &drive.path);
        }
        let vm_str = serde_yaml::to_string(&unresolved).unwrap();
        std::fs::write(path, vm_str)?;
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, SchemaWrite, SchemaRead)]
pub struct NetworkDevice {
    pub mac: String,
    pub master: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, SchemaWrite, SchemaRead)]
pub struct TapInterface {
    pub device: NetworkDevice,
    pub ifname: String,
}

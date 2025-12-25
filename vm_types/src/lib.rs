use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

pub mod utils;

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
    let base_path = absolute_base.parent().unwrap_or(Path::new("."));
    let resolved_path = base_path.join(relative.as_ref());
    resolved_path.to_string_lossy().to_string()
}

fn unresolve<P: AsRef<Path>, C: AsRef<Path>>(base: P, relative: C) -> String {
    match relative.as_ref().strip_prefix(&base) {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(_) => relative.as_ref().to_string_lossy().to_string(),
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OVMF {
    pub code: String,
    pub vars: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct API {
    pub groups: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub kvm: KVM,
    pub ovmf: OVMF,
    pub api: API,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KVM {
    pub bin: String,
    pub img: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let config_str = std::fs::read_to_string(path)?;
        let mut config: Config = serde_yaml::from_str(&config_str)?;
        config.kvm.bin = resolve(path, &config.kvm.bin);
        config.ovmf.code = resolve(path, &config.ovmf.code);
        config.ovmf.vars = resolve(path, &config.ovmf.vars);
        Ok(config)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Hardware {
    pub memory: u32,
    pub vcpu: u32,
    pub ovmf: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VNC {
    pub display: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VirtualMachine {
    pub name: String,
    pub hardware: Hardware,
    pub vnc: VNC,
    pub drives: HashMap<String, Drive>,
    pub networks: HashMap<String, NetworkInterface>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum MediaType {
    #[serde(rename = "cd")]
    Cdrom,
    #[serde(rename = "hd")]
    Disk,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct IdeDevice {
    pub media_type: MediaType,
    pub boot_index: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct VirtioBlkDevice {
    pub boot_index: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum DriveDevice {
    #[serde(rename = "ide")]
    Ide(IdeDevice),
    #[serde(rename = "virtio-blk")]
    VirtioBlk(VirtioBlkDevice)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NetworkDevice {
    pub mac: String,
    pub master: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TapInterface {
    pub device: NetworkDevice,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(tag = "type")]
pub enum NetworkInterface {
    #[serde(rename = "tap")]
    Tap(TapInterface),
}

impl<'a> NetworkInterface {
    pub fn get_network_device(&'a self) -> &'a NetworkDevice {
        match self {
            NetworkInterface::Tap(tap_interface) => &tap_interface.device,
        }
    }
}

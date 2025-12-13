use std::{collections::HashMap, path::Path};

use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Failed to load configuration: {0}")]
    TOML(#[from] toml::de::Error),
    #[error("Configuration file not found at path: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub kvm: KVM,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct KVM {
    pub bin: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let config_str = std::fs::read_to_string(path).map_err(ConfigError::from)?;
        let config: Config = toml::from_str(&config_str).map_err(ConfigError::from)?;
        Ok(config)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Hardware {
    pub memory: u32,
    pub vcpu: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct VirtualMachine {
    pub name: String,
    pub hardware: Hardware,
    pub drives: HashMap<String, Drive>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum IdeType {
    #[serde(rename = "cd")]
    Cdrom,
    #[serde(rename = "hd")]
    Disk,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IdeDevice {
    pub ide_type: IdeType,
    pub boot_index: Option<u32>,
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum DriveDevice {
    #[serde(rename = "ide")]
    Ide(IdeDevice),
}


#[derive(Debug, Deserialize, Serialize)]
pub struct Drive {
    pub path: String,
    pub device: DriveDevice,
}

impl VirtualMachine {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref().to_path_buf();
        let vm_str = std::fs::read_to_string(path.clone())?;
        let mut vm: VirtualMachine = toml::from_str(&vm_str)?;
        for (_, drive) in vm.drives.iter_mut() {
            if Path::new(&drive.path).is_relative() {
                let base_path = path.clone().parent().unwrap_or(Path::new(".")).to_path_buf();
                drive.path = base_path.join(&drive.path).to_string_lossy().to_string();
            }
        }
        Ok(vm)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let vm_str = toml::to_string_pretty(self).unwrap();
        std::fs::write(path, vm_str)?;
        Ok(())
    }
}

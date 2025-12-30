use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct VmLaunchRequest {
    pub hostname: String,
    pub ovmf: bool,
    pub vcpu: u32,
    pub memory: u32,
    pub vnc: Option<String>,
    pub drives: Vec<DriveConfig>,
    pub networks: Vec<NetworkConfig>,
    
    pub pid_file: PathBuf,
    pub qmp_socket: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriveBus {
    Ide {
        media_type: DiskMediaKind,
        boot_index: Option<u32>,
    },
    VirtioBlk {
        boot_index: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriveConfig {
    pub id: String,
    pub path: String,
    pub drive_media: DriveBus,
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub id: String,
    pub mac: String,
    pub ifname: String,
    pub netdev_up_script: Option<PathBuf>,
    pub netdev_down_script: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiskMediaKind {
    Disk,
    Cdrom,
}

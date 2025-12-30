use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DiskMediaKind {
    Cdrom,
    Disk,
}

impl From<DiskMediaKind> for qemu::device::DiskMediaKind {
    fn from(kind: DiskMediaKind) -> Self {
        match kind {
            DiskMediaKind::Cdrom => qemu::device::DiskMediaKind::Cdrom,
            DiskMediaKind::Disk => qemu::device::DiskMediaKind::Disk,
        }
    }
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum DriveBus {
    Ide {
        media_type: DiskMediaKind,
        boot_index: Option<u32>,
    },
    VirtioBlk {
        boot_index: Option<u32>,
    },
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DriveConfig {
    pub id: String,
    pub path: String,
    pub drive_media: DriveBus,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NetworkConfig {
    pub id: String,
    pub mac: String,
    pub ifname: String,
    pub netdev_up_script: Option<PathBuf>,
    pub netdev_down_script: Option<PathBuf>,
}

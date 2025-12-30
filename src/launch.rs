use std::path::PathBuf;

use qemu::{KVM, device::DiskMediaKind};

use crate::Error;

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

#[derive(Debug, Clone)]
pub enum DriveBus {
    Ide {
        media_type: DiskMediaKind,
        boot_index: Option<u32>,
    },
    VirtioBlk {
        boot_index: Option<u32>,
    },
}

#[derive(Debug, Clone)]
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

struct VmRunner {
    kvm: PathBuf,
    ovmf_code: PathBuf,
    ovmf_vars: PathBuf,
}

impl VmRunner {
    pub fn new(kvm: impl Into<PathBuf>, ovmf_code: impl Into<PathBuf>, ovmf_vars: impl Into<PathBuf>) -> Self {
        Self { kvm: kvm.into(), ovmf_code: ovmf_code.into(), ovmf_vars: ovmf_vars.into() }
    }

    fn args(&self, vm_request: &VmLaunchRequest) -> Vec<String> {
        let mut qemu = KVM::new(&self.kvm.to_string_lossy())
            .enable_kvm()
            .nodefaults()
            .qmp(&vm_request.qmp_socket)
            .pidfile(&vm_request.pid_file)
            .daemonize()
            .name(&vm_request.hostname)
            .memory(vm_request.memory)
            .smp(vm_request.vcpu)
            .virtio_vga();
        if vm_request.ovmf {
            qemu = qemu.ovmf(&self.ovmf_code, &self.ovmf_vars);
        }
        for drive in &vm_request.drives {
            qemu = qemu.drive(&drive.path, &drive.id);
            match &drive.drive_media {
                DriveBus::Ide { media_type, boot_index } => {
                    qemu = qemu.ide_device(&drive.id, *boot_index, &media_type.clone().into());
                },
                DriveBus::VirtioBlk { boot_index } => {
                    qemu = qemu.virtio_blk(&drive.id, *boot_index);
                },
            }
        }
        if let Some(vnc_display) = &vm_request.vnc {
            qemu = qemu.vnc(vnc_display, true);
        }
        for network in &vm_request.networks {
            qemu = qemu.netdev_tap(&network.id, network.netdev_up_script.as_ref(), network.netdev_down_script.as_ref(), &network.ifname);
            qemu = qemu.network_device(&network.id, &network.mac);
        }
        qemu.build()
    }

    pub async fn run_vm(&self, vm_request: &VmLaunchRequest) -> Result<(), Error> {
        let args = self.args(vm_request);
        let mut command = tokio::process::Command::new(&args[0]);
        command.args(&args[1..]);
        command.status().await?;
        Ok(())
    }
}

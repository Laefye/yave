use std::path::PathBuf;

use qemu::{KVM};
use vm_types::vm::{DriveBus, VmLaunchRequest};

use crate::Error;

pub struct VmRuntime {
    kvm: PathBuf,
    run_dir: PathBuf,
    ovmf_code: PathBuf,
    ovmf_vars: PathBuf,
    netdev_up_script: Option<PathBuf>,
    netdev_down_script: Option<PathBuf>,
}

impl VmRuntime {
    pub fn new(kvm: impl Into<PathBuf>, run_dir: impl Into<PathBuf>, ovmf_code: impl Into<PathBuf>, ovmf_vars: impl Into<PathBuf>, netdev_up_script: Option<PathBuf>, netdev_down_script: Option<PathBuf>) -> Self {
        Self { kvm: kvm.into(), run_dir: run_dir.into(), ovmf_code: ovmf_code.into(), ovmf_vars: ovmf_vars.into(), netdev_up_script, netdev_down_script }
    }

    fn args(&self, vm_request: &VmLaunchRequest) -> Vec<String> {
        let mut qemu = KVM::new(&self.kvm.to_string_lossy())
            .enable_kvm()
            .nodefaults()
            .qmp(&self.run_dir.join(&vm_request.id).with_added_extension("sock"))
            .pidfile(&self.run_dir.join(&vm_request.id).with_added_extension("pid"))
            .daemonize()
            .name(&vm_request.hostname)
            .memory(vm_request.memory)
            .smp(vm_request.vcpu)
            .virtio_vga();
        if vm_request.ovmf {
            qemu = qemu.ovmf(&self.ovmf_code, &self.ovmf_vars);
        }
        for drive in &vm_request.drives {
            qemu = qemu.drive(&drive.id, &drive.path);
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
            qemu = qemu.netdev_tap(&network.id, self.netdev_up_script.as_ref(), self.netdev_down_script.as_ref(), &network.ifname);
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

    pub async fn shutdown_vm(&self, vm_request: &VmLaunchRequest) -> Result<(), Error> {
        let mut qmp = self.qmp_connect(vm_request).await?;
        qmp.invoke(qmp::types::InvokeCommand::quit()).await?;
        qmp.on_close().await?;
        Ok(())
    }

    pub async fn reboot_vm(&self, vm_request: &VmLaunchRequest) -> Result<(), Error> {
        let qmp = self.qmp_connect(vm_request).await?;
        qmp.invoke(qmp::types::InvokeCommand::reboot()).await?;
        Ok(())
    }

    pub async fn qmp_connect(&self, vm_request: &VmLaunchRequest) -> Result<qmp::client::Client, Error> {
        let socket_path = self.run_dir.join(&vm_request.id).with_added_extension("sock");
        if !socket_path.exists() {
            return Err(Error::VMNotFound);
        }
        let qmp = qmp::client::Client::connect(&socket_path).await?;
        Ok(qmp)
    }

    pub async fn is_running(&self, vm_request: &VmLaunchRequest) -> Result<bool, Error> {
        match self.qmp_connect(vm_request).await {
            Ok(client) => client,
            Err(_) => return Ok(false),
        };
        Ok(true)
    }
}

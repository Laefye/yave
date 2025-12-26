use qemu::KVM;
use vm_types::{Config, DriveDevice, NetworkInterface, VirtualMachine};

use crate::{Error, yavecontext::YaveContextParams};

pub struct VmRunner {
    pub config: Config,
    pub vm: VirtualMachine,
}

impl VmRunner {
    pub fn new(config: Config, vm: VirtualMachine) -> Self {
        Self { config, vm }
    }

    fn build_drives(mut qemu: KVM, vm: &VirtualMachine) -> KVM {
        for (id, drive) in &vm.drives {
            qemu = qemu.drive(id, &drive.path);
            qemu = match &drive.device {
                DriveDevice::Ide(ide_device) => {
                    qemu.ide_device(id, ide_device.boot_index, &ide_device.media_type)
                },
                DriveDevice::VirtioBlk(virtio_blk_device) => {
                    qemu.virtio_blk(id, virtio_blk_device.boot_index)
                },
            }
        }
        qemu
    }

    pub async fn create_qmp(vm_config: &VirtualMachine, paths: &YaveContextParams) -> Result<qmp::client::Client, Error> {
        let socket_path = paths.with_vm_sock(&vm_config.name);
        let qmp = qmp::client::Client::connect(&socket_path).await?;
        Ok(qmp)
    }

    fn add_uefi(mut qemu: KVM, vm: &VirtualMachine, config: &Config) -> KVM {
        if let Some(true) = vm.hardware.ovmf {
            let code = config.ovmf.code.clone();
            let vars = config.ovmf.vars.clone();
            qemu = qemu.ovmf(code, vars);
        }
        qemu
    }

    fn add_vnc(qemu: KVM, vm: &VirtualMachine) -> KVM {
        if let Some(vnc) = &vm.vnc {
            qemu.vnc(&vnc.display, true)
        } else {
            qemu
        }
    }
    
    fn add_networks(mut qemu: KVM, vm: &VirtualMachine, paths: &YaveContextParams) -> KVM {
        for (id, net) in &vm.networks {
            match net {
                NetworkInterface::Tap(tap) => {
                    qemu = qemu.netdev_tap(id, Some(&paths.net_script_up), Some(&paths.net_script_down));
                    qemu = qemu.network_device(id, &tap.device.mac);
                },
            }
        }
        qemu
    }

    fn qemu_command(&self, paths: &YaveContextParams) -> Result<Vec<String>, Error> {
        let mut qemu = KVM::new(&self.config.cli.bin)
            .enable_kvm()
            .qmp(&paths.with_vm_sock(&self.vm.name))
            .daemonize()
            .name(&self.vm.name)
            .memory(self.vm.hardware.memory)
            .smp(self.vm.hardware.vcpu)
            .virtio_vga()
            .nodefaults();
        qemu = qemu.pidfile(&paths.with_vm_pid(&self.vm.name));
        qemu = Self::build_drives(qemu, &self.vm);
        qemu = Self::add_uefi(qemu, &self.vm, &self.config);
        qemu = Self::add_vnc(qemu, &self.vm);
        qemu = Self::add_networks(qemu, &self.vm, &paths);
        Ok(qemu.build())
    }

    pub async fn run(&self, paths: &YaveContextParams) -> Result<(), Error> {
        let args = self.qemu_command(paths)?;
        let mut command = tokio::process::Command::new(&args[0]);
        command.env(&paths.vm_name_env_variable, &self.vm.name);
        command.args(&args[1..]);
        command.status().await?;
        Ok(())
    }
}

use qemu::KVM;
use vm_types::{Config, DriveDevice, NetworkInterface, VirtualMachine};

use crate::{Error, contexts::{vm::VirtualMachineContext, yave::NetdevScripts}, yavecontext::YaveContextParams};

pub struct VmRunner<'a> {
    pub context: &'a VirtualMachineContext,
}

impl<'a> VmRunner<'a> {
    pub fn new(context: &'a VirtualMachineContext) -> Self {
        Self { context }
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
        qemu.vnc(&vm.vnc.display, true)
    }
    
    fn add_networks(mut qemu: KVM, vm: &VirtualMachine, netdev_scripts: &NetdevScripts) -> KVM {
        for (id, net) in &vm.networks {
            match net {
                NetworkInterface::Tap(tap) => {
                    qemu = qemu.netdev_tap(id, Some(&netdev_scripts.up), Some(&netdev_scripts.down));
                    qemu = qemu.network_device(id, &tap.device.mac);
                },
            }
        }
        qemu
    }

    fn get_qemu_command(&self) -> Result<Vec<String>, Error> {
        let mut qemu = KVM::new(&self.context.yave_context().config()?.cli.bin)
            .enable_kvm()
            .qmp(&self.context.qmp_socket())
            .daemonize()
            .name(&self.context.vm_config()?.name)
            .memory(self.context.vm_config()?.hardware.memory)
            .smp(self.context.vm_config()?.hardware.vcpu)
            .virtio_vga()
            .nodefaults();
        qemu = qemu.pidfile(&self.context.pid_file());
        qemu = Self::build_drives(qemu, &self.context.vm_config()?);
        qemu = Self::add_uefi(qemu, &self.context.vm_config()?, &self.context.yave_context().config()?);
        qemu = Self::add_vnc(qemu, &self.context.vm_config()?);
        qemu = Self::add_networks(qemu, &self.context.vm_config()?, self.context.yave_context().netdev_scripts());
        Ok(qemu.build())
    }

    pub async fn run(&self) -> Result<(), Error> {
        let args = self.get_qemu_command()?;
        let mut command = tokio::process::Command::new(&args[0]);
        command.env("YAVE_VM_NAME".to_string(), &self.context.vm_config()?.name);
        command.args(&args[1..]);
        command.status().await?;
        Ok(())
    }
}

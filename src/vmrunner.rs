use qemu::KVM;
use vm_types::{Config, DriveDevice, VirtualMachine};

use crate::{Error, contexts::{vm::VirtualMachineContext, yave::NetdevScripts}};

pub struct VmRunner<'a> {
    pub context: &'a VirtualMachineContext,
    pub vm_override: Option<VirtualMachine>,
}

impl<'a> VmRunner<'a> {
    pub fn new(context: &'a VirtualMachineContext) -> Self {
        Self { context, vm_override: None }
    }

    pub fn with_vm(mut self, vm: VirtualMachine) -> Self {
        self.vm_override = Some(vm);
        self
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

    fn get_vm(&self) -> Result<VirtualMachine, Error> {
        if let Some(vm) = &self.vm_override {
            Ok(vm.clone())
        } else {
            Ok(self.context.vm_config()?.clone())
        }
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
            qemu = qemu.netdev_tap(id, Some(&netdev_scripts.up), Some(&netdev_scripts.down));
            qemu = qemu.network_device(id, &net.device.mac);
        }
        qemu
    }

    fn get_qemu_command(&self) -> Result<Vec<String>, Error> {
        let vm = self.get_vm()?;
        let mut qemu = KVM::new(&self.context.yave_context().config()?.cli.bin)
            .enable_kvm()
            .qmp(&self.context.qmp_socket())
            .daemonize()
            .name(&vm.name)
            .memory(vm.hardware.memory)
            .smp(vm.hardware.vcpu)
            .virtio_vga()
            .nodefaults();
        qemu = qemu.pidfile(&self.context.pid_file());
        qemu = Self::build_drives(qemu, &vm);
        qemu = Self::add_uefi(qemu, &vm, &self.context.yave_context().config()?);
        qemu = Self::add_vnc(qemu, &vm);
        qemu = Self::add_networks(qemu, &vm, self.context.yave_context().netdev_scripts());
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

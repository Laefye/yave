use std::path::{Path, PathBuf};

use qemu::KVM;
use qmp::types::InvokeCommand;
use tokio::process::Command;
use vm_types::{Config, DriveDevice, NetworkInterface, VirtualMachine};

use crate::{Error, yavecontext::{YaveContext, YaveContextParams}};

pub struct VmContext {
    paths: YaveContextParams,
    vm_config_path: PathBuf,
}

impl VmContext {
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

    fn qemu_command(&self) -> Result<Vec<String>, Error> {
        let config = self.yave_context().config()?;
        let vm_config = self.vm_config()?;

        let mut qemu = KVM::new(&config.kvm.bin)
            .enable_kvm()
            .qmp(&self.paths.with_vm_sock(&vm_config.name))
            .pidfile(&self.paths.run_path.join(&vm_config.name).with_added_extension("pid"))
            .daemonize()
            .name(&vm_config.name)
            .memory(vm_config.hardware.memory)
            .smp(vm_config.hardware.vcpu)
            .virtio_vga()
            .nodefaults();
        qemu = Self::build_drives(qemu, &vm_config);
        qemu = Self::add_uefi(qemu, &vm_config, &config);
        qemu = Self::add_vnc(qemu, &vm_config);
        qemu = Self::add_networks(qemu, &vm_config, &self.paths);
        Ok(qemu.build())
    }

    pub fn new<P: AsRef<Path>>(config_path: YaveContextParams, vm_config_path: P) -> Self {
        Self {
            paths: config_path,
            vm_config_path: vm_config_path.as_ref().to_path_buf(),
        }
    }

    pub fn yave_context(&self) -> YaveContext {
        YaveContext::new(self.paths.clone())
    }
    
    pub fn vm_config(&self) -> Result<VirtualMachine, Error> {
        Ok(VirtualMachine::load(&self.vm_config_path)?)
    }

    pub async fn run(&self) -> Result<(), Error> {
        let vm_config = self.vm_config()?;
        let args = self.qemu_command()?;
        let mut command = Command::new(&args[0]);
        command.env(self.paths.vm_name_env_variable.clone(), &vm_config.name);
        command.args(&args[1..]);
        command.status().await?;
        let qmp = Self::create_qmp(&vm_config, &self.paths).await?;
        qmp.invoke(InvokeCommand::set_vnc_password(&vm_config.vnc.password)).await?;
        Ok(())
    }

    pub async fn connect_qmp(&self) -> Result<qmp::client::Client, Error> {
        let vm_config = self.vm_config()?;
        let qmp = Self::create_qmp(&vm_config, &self.paths).await?;
        Ok(qmp)
    }

    pub async fn create_qmp(vm_config: &VirtualMachine, paths: &YaveContextParams) -> Result<qmp::client::Client, Error> {
        let socket_path = paths.with_vm_sock(&vm_config.name);
        let qmp = qmp::client::Client::connect(&socket_path).await?;
        Ok(qmp)
    }
}

use std::path::{Path, PathBuf};

use qemu::KVM;
use qmp::types::InvokeCommand;
use tokio::process::Command;
use vm_types::{Config, DriveDevice, NetworkInterface, VirtualMachine};

use crate::{Error, yavecontext::{YaveContext, YaveContextParams}};

pub struct VmContext {
    params: YaveContextParams,
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
            .qmp(&self.params.with_vm_sock(&vm_config.name))
            .pidfile(&self.params.with_vm_pid(&vm_config.name))
            .daemonize()
            .name(&vm_config.name)
            .memory(vm_config.hardware.memory)
            .smp(vm_config.hardware.vcpu)
            .virtio_vga()
            .nodefaults();
        qemu = Self::build_drives(qemu, &vm_config);
        qemu = Self::add_uefi(qemu, &vm_config, &config);
        qemu = Self::add_vnc(qemu, &vm_config);
        qemu = Self::add_networks(qemu, &vm_config, &self.params);
        Ok(qemu.build())
    }

    pub fn new<P: AsRef<Path>>(config_path: YaveContextParams, vm_config_path: P) -> Self {
        Self {
            params: config_path,
            vm_config_path: vm_config_path.as_ref().to_path_buf(),
        }
    }

    pub fn yave_context(&self) -> YaveContext {
        YaveContext::new(self.params.clone())
    }
    
    pub fn vm_config(&self) -> Result<VirtualMachine, Error> {
        Ok(VirtualMachine::load(&self.vm_config_path)?)
    }

    pub async fn run(&self) -> Result<(), Error> {
        let vm_config = self.vm_config()?;
        if Self::_is_running(&vm_config, &self.params).await? {
            return Err(Error::VMRunning(vm_config.name));
        }
        let args = self.qemu_command()?;
        let mut command = Command::new(&args[0]);
        command.env(self.params.vm_name_env_variable.clone(), &vm_config.name);
        command.args(&args[1..]);
        command.status().await?;
        let qmp = Self::create_qmp(&vm_config, &self.params).await?;
        qmp.invoke(InvokeCommand::set_vnc_password(&vm_config.vnc.password)).await?;
        Ok(())
    }

    pub async fn connect_qmp(&self) -> Result<qmp::client::Client, Error> {
        let vm_config = self.vm_config()?;
        if !Self::_is_running(&vm_config, &self.params).await? {
            return Err(Error::VMNotRunning(vm_config.name));
        }
        let qmp = Self::create_qmp(&vm_config, &self.params).await?;
        Ok(qmp)
    }

    async fn create_qmp(vm_config: &VirtualMachine, paths: &YaveContextParams) -> Result<qmp::client::Client, Error> {
        let socket_path = paths.with_vm_sock(&vm_config.name);
        let qmp = qmp::client::Client::connect(&socket_path).await?;
        Ok(qmp)
    }

    async fn _is_running(vm: &VirtualMachine, paths: &YaveContextParams) -> Result<bool, Error> {
        let pid_path = paths.with_vm_pid(&vm.name);
        let pid_str = match std::fs::read_to_string(pid_path) {
            Ok(s) => s,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(e) => return Err(Error::IO(e)),
        };
        let pid: i32 = pid_str.trim().parse().unwrap_or(0);
        match nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None) {
            Ok(_) => Ok(true),
            Err(nix::errno::Errno::ESRCH) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn is_running(&self) -> Result<bool, Error> {
        let vm_config = self.vm_config()?;
        Self::_is_running(&vm_config, &self.params).await
    }
}

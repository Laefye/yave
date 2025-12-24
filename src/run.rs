use std::path::{Path, PathBuf};

use qemu::KVM;
use tokio::process::Command;
use vm_types::{Config, DriveDevice, NetworkInterface, VirtualMachine};

pub struct RunFactory<'a> {
    socket: PathBuf,
    pidfile: PathBuf,
    net_script_up: PathBuf,
    net_script_down: PathBuf,
    config: &'a Config,
    vm: &'a VirtualMachine,
    env_name: String,
}

fn create_parent_dir(path: &Path) -> Result<(), std::io::Error> {
    std::fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))?;
    Ok(())
}

impl<'a> RunFactory<'a> {
    pub fn new<SocketPath, NetScriptUpPath, NetScriptDownPath>(
        run_dir: SocketPath,
        net_script_up: NetScriptUpPath,
        net_script_down: NetScriptDownPath,
        vm: &'a VirtualMachine,
        config: &'a Config,
        env_name: &str,
    ) -> Self 
    where 
        SocketPath: AsRef<std::path::Path>,
        NetScriptUpPath: AsRef<std::path::Path>,
        NetScriptDownPath: AsRef<std::path::Path>,
    {
        Self {
            socket: run_dir.as_ref().to_path_buf().join(format!("{}.sock", vm.name)),
            pidfile: run_dir.as_ref().to_path_buf().join(format!("{}.pid", vm.name)),
            net_script_up: net_script_up.as_ref().to_path_buf(),
            net_script_down: net_script_down.as_ref().to_path_buf(),
            config,
            vm,
            env_name: env_name.to_string(),
        }
    }

    fn build_drives(&self, mut qemu: KVM) -> KVM {
        for (id, drive) in &self.vm.drives {
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

    pub fn get_pidfile_path(&self) -> &PathBuf {
        &self.pidfile
    }

    pub fn get_socket_path(&self) -> &PathBuf {
        &self.socket
    }

    fn add_uefi(&self, mut qemu: KVM) -> KVM {
        if let Some(true) = self.vm.hardware.ovmf {
            let code = self.config.ovmf.code.clone();
            let vars = self.config.ovmf.vars.clone();
            qemu = qemu.ovmf(code, vars);
        }
        qemu
    }

    fn add_vnc(&self, qemu: KVM) -> KVM {
        qemu.vnc(&self.vm.vnc.port, true)
    }
    
    fn add_networks(&self, mut qemu: KVM) -> KVM {
        for (id, net) in &self.vm.networks {
            match net {
                NetworkInterface::Tap(tap) => {
                    qemu = qemu.netdev_tap(id, Some(&self.net_script_up), Some(&self.net_script_down));
                    qemu = qemu.network_device(id, &tap.device.mac);
                },
            }
        }
        qemu
    }

    fn build_qemu_command(&self) -> Vec<String> {
        let mut qemu = KVM::new(&self.config.kvm.bin.clone())
            .enable_kvm()
            .qmp(&self.socket)
            .pidfile(&self.pidfile)
            .daemonize()
            .name(&self.vm.name)
            .memory(self.vm.hardware.memory)
            .smp(self.vm.hardware.vcpu)
            .virtio_vga()
            .nodefaults();
        qemu = self.build_drives(qemu);
        qemu = self.add_uefi(qemu);
        qemu = self.add_vnc(qemu);
        qemu = self.add_networks(qemu);
        qemu.build()
    }

    pub async fn run(&self) -> Result<(), std::io::Error> {
        create_parent_dir(self.get_socket_path())?;
        create_parent_dir(self.get_pidfile_path())?;
        let args = self.build_qemu_command();
        let mut command = Command::new(&args[0]);
        command.env(&self.env_name, &self.vm.name);
        command.args(&args[1..]);
        command.status().await?;
        Ok(())
    }
}

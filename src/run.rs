use std::path::PathBuf;

use qemu::QEMU;
use crate::config::{Config, DriveDevice, VirtualMachine};

pub struct RunFactory<'a> {
    socket: PathBuf,
    pidfile: PathBuf,
    net_script_up: PathBuf,
    net_script_down: PathBuf,
    config: &'a Config,
    vm: &'a VirtualMachine,
}

impl<'a> RunFactory<'a> {
    pub fn new<SocketPath, PidfilePath, NetScriptUpPath, NetScriptDownPath>(socket: SocketPath, pidfile: PidfilePath, net_script_up: NetScriptUpPath, net_script_down: NetScriptDownPath, vm: &'a VirtualMachine, config: &'a Config) -> Self 
    where 
        SocketPath: AsRef<std::path::Path>,
        PidfilePath: AsRef<std::path::Path>,
        NetScriptUpPath: AsRef<std::path::Path>,
        NetScriptDownPath: AsRef<std::path::Path>,
    {
        Self {
            socket: socket.as_ref().to_path_buf().join(format!("{}.sock", vm.name)),
            pidfile: pidfile.as_ref().to_path_buf().join(format!("{}.pid", vm.name)),
            net_script_up: net_script_up.as_ref().to_path_buf(),
            net_script_down: net_script_down.as_ref().to_path_buf(),
            config,
            vm,
        }
    }

    fn build_drives(&self, mut qemu: QEMU) -> QEMU {
        for (id, drive) in &self.vm.drives {
            qemu = qemu.drive(id, &drive.path);
            qemu = match &drive.device {
                DriveDevice::Ide(ide_device) => {
                    qemu.ide_device(id, ide_device.boot_index, match ide_device.media_type {
                        crate::config::MediaType::Disk => qemu::device::MediaType::Disk,
                        crate::config::MediaType::Cdrom => qemu::device::MediaType::Cdrom,
                    })
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

    fn add_uefi(&self, mut qemu: QEMU) -> QEMU {
        if let Some(true) = self.vm.hardware.ovmf {
            let code = self.config.ovmf.code.clone();
            let vars = self.config.ovmf.vars.clone();
            qemu = qemu.ovmf(code, vars);
        }
        qemu
    }

    fn add_vnc(&self, qemu: QEMU) -> QEMU {
        qemu.vnc(&self.vm.vnc.port, true)
    }
    
    fn add_networks(&self, mut qemu: QEMU) -> QEMU {
        for (id, net) in &self.vm.networks {
            match net {
                crate::config::NetworkInterface::Tap(tap) => {
                    qemu = qemu.netdev_tap(id, &tap.ifname, Some(&self.net_script_up), Some(&self.net_script_down));
                    qemu = qemu.network_device(id, &tap.device.mac);
                },
            }
        }
        qemu
    }

    pub fn build_qemu_command(&self) -> Vec<String> {
        let mut qemu = QEMU::new(&self.config.kvm.bin.clone())
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
}

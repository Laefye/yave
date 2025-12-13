use std::path::PathBuf;

use crate::{config::{Config, DriveDevice, VirtualMachine}, qemu::QEMU};

pub struct RunFactory<'a> {
    socket: PathBuf,
    pidfile: PathBuf,
    vm: &'a VirtualMachine,
    config: &'a Config,
}

impl<'a> RunFactory<'a> {
    pub fn new<SocketPath: AsRef<std::path::Path>, PidfilePath: AsRef<std::path::Path>>(socket: SocketPath, pidfile: PidfilePath, vm: &'a VirtualMachine, config: &'a Config) -> Self {
        Self {
            socket: socket.as_ref().to_path_buf().join(format!("{}.sock", vm.name)),
            pidfile: pidfile.as_ref().to_path_buf().join(format!("{}.pid", vm.name)),
            vm,
            config,
        }
    }

    fn build_drives(&self, mut qemu: QEMU) -> QEMU {
        for (id, drive) in &self.vm.drives {
            qemu = qemu.drive(id, &drive.path);
            qemu = match &drive.device {
                DriveDevice::Ide(ide_device) => {
                    qemu.ide_device(id, ide_device.boot_index, match ide_device.ide_type {
                        crate::config::IdeType::Disk => crate::qemu::device::IdeType::Disk,
                        crate::config::IdeType::Cdrom => crate::qemu::device::IdeType::Cdrom,
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

    pub fn build_qemu_command(&self) -> Vec<String> {
        let mut qemu = QEMU::new(&self.config.kvm.bin.clone())
            .qmp(&self.socket)
            .pidfile(&self.pidfile)
            .daemonize()
            .name(&self.vm.name)
            .memory(self.vm.hardware.memory)
            .smp(self.vm.hardware.vcpu);
        qemu = self.build_drives(qemu);
        qemu = self.add_uefi(qemu);
        qemu = self.add_vnc(qemu);
        qemu.build()
    }
}

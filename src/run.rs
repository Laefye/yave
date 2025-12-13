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

    pub fn build_qemu_command(&self) -> Vec<String> {
        let qemu = QEMU::new(&self.config.kvm.bin.clone())
            .qmp(&self.socket)
            .pidfile(&self.pidfile)
            .daemonize()
            .vnc(":1", true)
            .name(&self.vm.name)
            .memory(self.vm.hardware.memory)
            .smp(self.vm.hardware.vcpu);
        let qemu = self.build_drives(qemu);
        qemu.build()
    }
}

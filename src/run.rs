use std::{collections::HashMap, path::PathBuf};

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
        let mut sata_indices = HashMap::new();
        for key in self.vm.drives.values().filter(|x| matches!(x.device, DriveDevice::Ide(_))) {
            if let DriveDevice::Ide(ide_device) = &key.device {
                if let Some(sata_bus) = &ide_device.sata_bus && !sata_indices.contains_key(sata_bus) {
                    sata_indices.insert(sata_bus.clone(), 0);
                    qemu = qemu.achi9_controller(sata_bus);
                }
            }
        }

        for (id, drive) in &self.vm.drives {
            qemu = qemu.drive(id, &drive.path);
            qemu = match &drive.device {
                DriveDevice::Ide(ide_device) => {
                    let bus = if ide_device.sata_bus.is_some() {
                        let bus = ide_device.sata_bus.as_ref().unwrap();
                        let index = sata_indices.get_mut(bus).unwrap();
                        let current_index = *index;
                        *index += 1;
                        Some(format!("{}.{}", bus, current_index))
                    } else {
                        None
                    };

                    qemu.ide_device(id, ide_device.boot_index, match ide_device.media_type {
                        crate::config::MediaType::Disk => crate::qemu::device::MediaType::Disk,
                        crate::config::MediaType::Cdrom => crate::qemu::device::MediaType::Cdrom,
                    }, bus.as_deref())
                },
                DriveDevice::Nvme(nvme_device) => {
                    qemu.nvme_device(id, nvme_device.boot_index, &nvme_device.serial)
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
                    qemu = qemu.netdev_tap(id, &tap.ifname, None, None);
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

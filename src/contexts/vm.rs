use std::{collections::HashMap, path::PathBuf};

use vm_types::VirtioBlkDevice;

use crate::{Error, db::{allocate_vnc_display, get_vm, insert_vm}, tools};

use super::yave::YaveContext;

#[derive(Debug, Clone)]
pub struct VirtualMachineContext {
    yave_context: YaveContext,
    name: String,
}

impl VirtualMachineContext {
    pub(super) fn new(yave_context: YaveContext, name: &str) -> Self {
        Self {
            yave_context,
            name: name.to_string(),
        }
    }

    pub fn yave_context(&self) -> &YaveContext {
        &self.yave_context
    }

    pub fn vm(&self) -> Result<vm_types::VirtualMachine, crate::Error> {
        let db = self.yave_context.database()?;
        let vm = get_vm(&db, &self.name)?
            .ok_or_else(|| crate::Error::VMNotFound(self.name.clone()))?;
        Ok(vm)
    }

    pub fn pid_file(&self) -> PathBuf {
        self.yave_context
            .run_path()
            .join(&self.name)
            .with_extension("pid")
    }

    pub fn qmp_socket(&self) -> PathBuf {
        self.yave_context
            .run_path()
            .join(&self.name)
            .with_extension("sock")
    }

    pub async fn connect_qmp(&self) -> Result<qmp::client::Client, Error> {
        if !self.is_running().await? {
            return Err(Error::VMNotRunning(self.vm()?.name));
        }
        let qmp = qmp::client::Client::connect(&self.qmp_socket()).await?;
        Ok(qmp)
    }
    
    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn is_running(&self) -> Result<bool, Error> {
        let pid_path = self.pid_file();
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
}

pub enum DriveOptions {
    Empty {
        size: u32,
    },
    From {
        size: Option<u32>,
        image: String,
    },
}

pub struct NetworkOptions {

}

pub struct VirtualMachineFactory {
    yave_context: YaveContext,
    name: String,
    vcpu: u32,
    memory: u32,
    drives: Vec<DriveOptions>,
    networks: Vec<NetworkOptions>,
}

impl VirtualMachineFactory {
    pub fn new(yave_context: &YaveContext, name: &str) -> Self {
        Self {
            yave_context: yave_context.clone(),
            name: name.to_string(),
            vcpu: 1,
            memory: 1024,
            drives: Vec::new(),
            networks: Vec::new(),
        }
    }
    pub fn vcpu(mut self, vcpu: u32) -> Self {
        self.vcpu = vcpu;
        self
    }
    pub fn memory(mut self, memory: u32) -> Self {
        self.memory = memory;
        self
    }
    pub fn drive(mut self, drive: DriveOptions) -> Self {
        self.drives.push(drive);
        self
    }

    pub fn network(mut self, network: NetworkOptions) -> Self {
        self.networks.push(network);
        self
    }

    pub async fn create(&self) -> Result<VirtualMachineContext, crate::Error> {
        let tap_table_path = self.yave_context.net_table();
        let mut tap_table = vm_types::NetTable::load(&tap_table_path)?;
        let vm_dir = self.yave_context.vm_dir(&self.name);
        let mut database = self.yave_context.database()?;
        std::fs::create_dir_all(&vm_dir)?;
        let mut vm = vm_types::VirtualMachine {
            name: self.name.clone(),
            hardware: vm_types::Hardware {
                memory: self.memory,
                vcpu: self.vcpu,
                ovmf: Some(true),
            },
            networks: HashMap::new(),
            drives: HashMap::new(),
            vnc: vm_types::VNC {
                display: allocate_vnc_display(&mut database, &self.name)?,
            },
        };

        for (i, drive) in self.drives.iter().enumerate() {
            let drive_id = format!("hd{}", i);
            let drive_path = vm_dir.join(format!("{}.img", drive_id));
            match drive {
                DriveOptions::Empty { size } => {
                    tools::QemuImg::new(&self.yave_context.config().cli.img)
                        .create(*size, &drive_path).await?;
                },
                DriveOptions::From { size, image } => {
                    std::fs::copy(&self.yave_context.storage_path().join(image).with_added_extension("img"), &drive_path)?;
                    if let Some(size) = size {
                        tools::QemuImg::new(&self.yave_context.config().cli.img)
                            .resize(*size, &drive_path).await?;
                    }
                },
            }
            vm.drives.insert(drive_id, vm_types::Drive {
                device: vm_types::DriveDevice::VirtioBlk(VirtioBlkDevice {
                    boot_index: Some(i as u32 + 1)
                }),
                path: drive_path.to_string_lossy().to_string(),
            });
        }

        for (i, _net) in self.networks.iter().enumerate() {
            let tap_ifname = tap_table.allocate_tap(&self.name);
            let net_id = format!("net{}", i);
            vm.networks.insert(net_id, vm_types::TapInterface {
                device: vm_types::NetworkDevice {
                    mac: vm_types::utils::get_mac(&tap_ifname),
                    master: None,
                },
                ifname: tap_ifname,
            });
        }

        insert_vm(&database, &self.name, &vm)?;
        Ok(VirtualMachineContext::new(self.yave_context.clone(), &self.name.to_string()))
    }
}

use std::{collections::HashMap, ffi::OsString, path::{Path, PathBuf}};

use vm_types::{Config, Drive, DriveDevice, Hardware, NetworkDevice, NetworkInterface, TapInterface, VNC, VNCTable, VirtioBlkDevice, VirtualMachine};

use crate::{Error, images::QemuImg, vmcontext::VmContext};

#[derive(Clone)]
pub struct YaveContextParams {
    pub storage_path: PathBuf,
    pub config_path: PathBuf,
    pub run_path: PathBuf,
    pub net_script_up: PathBuf,
    pub net_script_down: PathBuf,
    pub vm_ext: OsString,
    pub hd_ext: OsString,
    pub vm_config_name: OsString,
    pub vm_name_env_variable: String,
} 

impl YaveContextParams {
    pub fn with_vm<P: AsRef<Path>>(&self, name: P) -> PathBuf {
        self.storage_path.join(name).with_added_extension(&self.vm_ext)
    }

    pub fn with_vm_sock<P: AsRef<Path>>(&self, name: P) -> PathBuf {
        self.run_path.join(name).with_added_extension("sock")
    }

    pub fn with_vm_pid<P: AsRef<Path>>(&self, name: P) -> PathBuf {
        self.run_path.join(name).with_added_extension("pid")
    }
}

pub struct YaveContext {
    params: YaveContextParams,
}

pub enum CreateDriveOptions {
    Empty {
        size: u32,
    },
    FromStorage {
        image: String,
    }
} 

pub struct CreateVirtualMachineInput {
    name: String,
    hardware: Hardware,
    drives: Vec<CreateDriveOptions>,
}

impl CreateVirtualMachineInput {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            hardware: Hardware {
                memory: 1024,
                vcpu: 1,
                ovmf: Some(true),
            },
            drives: Vec::new(),
        }
    }
    
    pub fn drive(mut self, drive: CreateDriveOptions) -> Self{
        self.drives.push(drive);
        self
    }

    pub fn vcpu(mut self, vcpu: u32) -> Self {
        self.hardware.vcpu = vcpu;
        self
    }

    pub fn memory(mut self, memory: u32) -> Self {
        self.hardware.memory = memory;
        self
    }
}

impl YaveContext {
    pub fn new(pathes: YaveContextParams) -> Self {
        Self {
            params: pathes,
        }
    }

    pub async fn create_vm(&self, input: CreateVirtualMachineInput) -> Result<VmContext, Error> {
        let mut vnc_table = self.vnc_table()?;

        let mut vm = VirtualMachine {
            name: input.name.clone(),
            hardware: input.hardware,
            vnc: VNC {
                display: vnc_table.find_free_display(),
                password: "12345678".to_string(),
            },
            drives: HashMap::new(),
            networks: HashMap::new(),
        };

        let vm_path = self.params.with_vm(&input.name);
        let vm_config = vm_path.join(&self.params.vm_config_name);

        std::fs::create_dir_all(&vm_path)?;

        for (i, drive_option) in input.drives.iter().enumerate() {
            match drive_option {
                CreateDriveOptions::Empty { size } => {
                    let hd_id = format!("hd{}", i);
                    let hd_file = vm_path.join(&hd_id).with_added_extension(&self.params.hd_ext);
                    vm.drives.insert(hd_id, Drive {
                        path: hd_file.to_string_lossy().to_string(),
                        device: DriveDevice::VirtioBlk(VirtioBlkDevice {
                            boot_index: Some((i as u32) + 1),
                        })
                    });
                    QemuImg::new(self.config()?.kvm.img)
                        .run(*size, &hd_file).await?;
                },
                CreateDriveOptions::FromStorage { image } => {
                    let hd_id = format!("hd{}", i);
                    let hd_file = vm_path.join(&hd_id).with_added_extension(&self.params.hd_ext);
                    std::fs::copy(self.params.storage_path.join(&image).with_added_extension(&self.params.hd_ext), &hd_file)?;
                    vm.drives.insert(hd_id, Drive {
                        path: hd_file.to_string_lossy().to_string(),
                        device: DriveDevice::VirtioBlk(VirtioBlkDevice {
                            boot_index: Some((i as u32) + 1),
                        })
                    });
                },
            }
        }

        vm.networks.insert("net0".to_string(), NetworkInterface::Tap(
            TapInterface {
                device: NetworkDevice {
                    mac: vm_types::utils::get_mac(&input.name),
                    master: None,
                }
            } 
        ));
        
        
        vm.save(&vm_config)?;
        vnc_table.table.insert(vm.vnc.display.clone(), input.name.clone());
        self.update_vnc_table(&vnc_table)?;

        Ok(VmContext::new(
            self.params.clone(),
            &vm_config
        ))
    }

    pub fn config(&self) -> Result<Config, Error> {
        Ok(Config::load(&self.params.config_path)?)
    }

    pub fn open_vm(&self, name: &str) -> VmContext {
        let vm_config_path = self.params.with_vm(name).join(&self.params.vm_config_name);
        VmContext::new(
            self.params.clone(),
            &vm_config_path
        )
    }

    pub fn list(&self) -> Result<Vec<String>, Error> {
        let mut vms = Vec::new();
        for entry in std::fs::read_dir(&self.params.storage_path)? {
            let entry = entry?;
            if let Some(ext) = entry.path().extension() {
                if ext == self.params.vm_ext {
                    vms.push(entry.path().file_stem().unwrap().to_string_lossy().to_string());
                }
            }
        }
        Ok(vms)
    }

    pub fn vnc_table(&self) -> Result<VNCTable, Error> {
        Ok(VNCTable::load(
            &self.params.storage_path.join("vnc_table.yaml")
        )?)
    }

    pub fn update_vnc_table(&self, table: &VNCTable) -> Result<(), Error> {
        table.save(
            &self.params.storage_path.join("vnc_table.yaml")
        )?;
        Ok(())
    }
}

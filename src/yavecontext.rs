use std::{collections::HashMap, ffi::OsString, path::{Path, PathBuf}};

use vm_types::{Config, Drive, DriveDevice, Hardware, NetworkDevice, NetworkInterface, Preset, TapInterface, VNC, VNCTable, VirtioBlkDevice, VirtualMachine, cloudinit::{Chpasswd, CloudConfig, PowerState}};

use crate::{Error, installer::Installer, tools::QemuImg, vmcontext::OldVmContext};

#[derive(Clone)]
pub struct YaveContextParams {
    pub storage_path: PathBuf,
    pub config_path: PathBuf,
    pub run_path: PathBuf,
    pub net_script_up: PathBuf,
    pub net_script_down: PathBuf,
    pub vm_ext: OsString,
    pub hd_ext: OsString,
    pub preset_ext: OsString,
    pub vm_config_name: OsString,
    pub vm_name_env_variable: String,
    pub cloud_init_iso_name: OsString,
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

#[derive(Clone)]
pub struct YaveContext {
    params: YaveContextParams,
}

pub enum CreateDriveOptions {
    Empty {
        size: u32,
    },
    FromStorage {
        image: String,
    },
    FromPreset {
        size: u32,
        preset: String,
    },
} 

pub struct Passwords {
    pub root: String,
    pub vnc: String,
}

pub struct CreateVirtualMachineInput {
    name: String,
    hostname: Option<String>,
    hardware: Hardware,
    drives: Vec<CreateDriveOptions>,
    passwords: Option<Passwords>,
}

impl CreateVirtualMachineInput {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            hostname: None,
            hardware: Hardware {
                memory: 1024,
                vcpu: 1,
                ovmf: Some(true),
            },
            drives: Vec::new(),
            passwords: None,
        }
    }

    pub fn hostname(mut self, hostname: &str) -> Self {
        self.hostname = Some(hostname.to_string());
        self
    }

    pub fn passwords(mut self, passwords: Passwords) -> Self {
        self.passwords = Some(passwords);
        self
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

    async fn create_cloud_init_config(&self, passwords: &Passwords, hostname: &str) -> Result<CloudConfig, Error> {
        let cloud_config = CloudConfig {
            hostname: hostname.to_string(),
            chpasswd: Chpasswd {
                expire: false,
            },
            power_state: PowerState::default(),
            password: passwords.root.clone(),
            ssh_pwauth: true,
        };
        Ok(cloud_config)
    }

    pub async fn create_vm(&self, input: CreateVirtualMachineInput) -> Result<OldVmContext, Error> {
        unimplemented!()
    }

    pub fn config(&self) -> Result<Config, Error> {
        Ok(Config::load(&self.params.config_path)?)
    }

    pub fn open_vm(&self, name: &str) -> Result<OldVmContext, Error> {
        let vm_config_path = self.params.with_vm(name).join(&self.params.vm_config_name);
        if !std::fs::exists(&self.params.with_vm(name))? {
            Err(Error::VMNotFound(name.into()))
        } else {
            Ok(OldVmContext::new(
                self.params.clone(),
                &vm_config_path
            ))
        }
        
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

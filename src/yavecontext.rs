use std::{collections::HashMap, ffi::OsString, path::{Path, PathBuf}};

use vm_types::{Config, Drive, DriveDevice, Hardware, NetworkDevice, NetworkInterface, Preset, TapInterface, VNC, VNCTable, VirtioBlkDevice, VirtualMachine, cloudinit::{Chpasswd, CloudConfig, PowerState}};

use crate::{Error, presetinstaller::PresetInstaller, tools::QemuImg, vmcontext::OldVmContext};

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
        let mut vnc_table = self.vnc_table()?;

        let mut vm = VirtualMachine {
            name: input.name.clone(),
            hardware: input.hardware,
            vnc: VNC {
                display: vnc_table.find_free_display(),
                password: input.passwords.as_ref().map_or("12345678".to_string(), |p| p.vnc.clone()),
            },
            drives: HashMap::new(),
            networks: HashMap::new(),
        };

        let vm_path = self.params.with_vm(&input.name);
        let vm_config = vm_path.join(&self.params.vm_config_name);
        let config = self.config()?;

        std::fs::create_dir_all(&vm_path)?;

        let mut presets_to_install = Vec::new();

        for (i, drive_option) in input.drives.iter().enumerate() {
            match drive_option {
                CreateDriveOptions::Empty { size } => {
                    let hd_id = format!("hd{}", i);
                    let hd_file = vm_path.join(&hd_id).with_added_extension(&self.params.hd_ext);
                    vm.drives.insert(hd_id, Drive {
                        path: hd_file.to_string_lossy().to_string(),
                        device: DriveDevice::VirtioBlk(VirtioBlkDevice {
                            boot_index: Some( i as u32),
                        })
                    });
                    QemuImg::new(self.config()?.cli.img)
                        .create(*size, &hd_file).await?;
                },
                CreateDriveOptions::FromStorage { image } => {
                    let hd_id = format!("hd{}", i);
                    let hd_file = vm_path.join(&hd_id).with_added_extension(&self.params.hd_ext);
                    std::fs::copy(self.params.storage_path.join(&image).with_added_extension(&self.params.hd_ext), &hd_file)?;
                    vm.drives.insert(hd_id, Drive {
                        path: hd_file.to_string_lossy().to_string(),
                        device: DriveDevice::VirtioBlk(VirtioBlkDevice {
                            boot_index: Some(i as u32),
                        })
                    });
                },
                CreateDriveOptions::FromPreset { size, preset } => {
                    let hd_id = format!("hd{}", i);
                    let hd_file = vm_path.join(&hd_id).with_added_extension(&self.params.hd_ext);
                    let preset = Preset::load(
                        &self.params.storage_path.join(preset).with_added_extension(&self.params.preset_ext).join("config.yaml")
                    )?;
                    QemuImg::new(self.config()?.cli.img)
                        .convert(&preset.cloudimg, &hd_file).await?;
                    QemuImg::new(self.config()?.cli.img)
                        .resize(*size, &hd_file).await?;
                    let cloud_init_config = self.create_cloud_init_config(
                        input.passwords.as_ref().unwrap(),
                        &input.hostname.as_ref().unwrap_or(&input.name),
                    ).await?;
                    presets_to_install.push((hd_file.clone(), cloud_init_config));
                    vm.drives.insert(hd_id, Drive {
                        path: hd_file.to_string_lossy().to_string(),
                        device: DriveDevice::VirtioBlk(VirtioBlkDevice {
                            boot_index: Some(i as u32),
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

        for (hd_file, cloud_init_config) in presets_to_install {
            let preset_installer = PresetInstaller::new(vm.clone(), &hd_file, cloud_init_config);
            preset_installer.install(&config, &self.params).await?;
        }

        Ok(OldVmContext::new(
            self.params.clone(),
            &vm_config
        ))
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

use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use vm_types::{Config, Drive, DriveDevice, Hardware, NetworkDevice, NetworkInterface, TapInterface, VNC, VirtioBlkDevice, VirtualMachine};

use crate::{DefaultFacade, Error, Facade, constants::{get_config_path, get_net_script, get_run_path, get_vm_config_path, get_vm_env_variable, get_vminstance_extension}, images::Images, vmcontext::VmContext};

#[derive(Debug, Serialize, Deserialize)]
pub enum InputOperatingSystem {
    Empty,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VirtualMachineCreateInput {
    pub name: String,
    pub vcpu: u32,
    pub memory: u32,
    pub capacity: u32,
    pub os: InputOperatingSystem,
}

fn make_config(input: &VirtualMachineCreateInput) -> VirtualMachine {
    let mut drives = HashMap::new();
    drives.insert("hd0".to_string(), Drive { path: "hd0.qcow2".to_string(), device: DriveDevice::VirtioBlk(VirtioBlkDevice { boot_index: Some(1) }) });
    let mut networks = HashMap::new();
    networks.insert("net0".to_string(), NetworkInterface::Tap(TapInterface {
        device: NetworkDevice {
            mac: "52:54:00:12:34:56".to_string(),
            master: None,
        }
    }));
    VirtualMachine {
        vnc: VNC { port: ":1".to_string(), password: "12345678".to_string() },
        name: input.name.clone(),
        drives,
        hardware: Hardware {
            memory: input.memory,
            vcpu: input.vcpu,
            ovmf: Some(true),
        },
        networks,
    }
}


#[async_trait]
impl Facade<VirtualMachineCreateInput> for DefaultFacade {
    type Output = ();

    async fn invoke(&self, input: VirtualMachineCreateInput) -> Result<Self::Output, Error> {
        let vm_dir = &get_vm_config_path().join(&format!("{}.vminstance", input.name));
        std::fs::create_dir_all(vm_dir)?;
        
        let vm_config = make_config(&input);
        println!("Saving config {:?}", vm_config);
        
        vm_config.save(vm_dir.join("config.yaml"))?;
        
        let config = Config::load(&get_config_path())?;
        let image = Images::new(config.kvm.img);
        image.run(input.capacity, vm_dir.join("hd0.qcow2")).await.expect("Ikd");

        Ok(())
    }
}

pub struct ListVirtualMachinesInput;

#[async_trait]
impl Facade<ListVirtualMachinesInput> for DefaultFacade {
    type Output = Vec<String>;
    
    async fn invoke(&self, _: ListVirtualMachinesInput) -> Result<Self::Output, Error> {
        let dir = std::fs::read_dir(&get_vm_config_path())?;
        
        let mut vms = vec![];

        for entry in dir {
            let entry = entry?;
            let file_name = entry.file_name().to_string_lossy().to_string();
            if let Some(file_name) = file_name.strip_suffix(&format!(".{}", get_vminstance_extension())) {
                vms.push(file_name.to_string());
            }
        }
        Ok(vms)
    }
}

pub struct RunVirtualMachinesInput {
    pub name: String,
}

#[async_trait]
impl Facade<RunVirtualMachinesInput> for DefaultFacade {
    type Output = ();
    
    async fn invoke(&self, run_virtual_machines_input: RunVirtualMachinesInput) -> Result<Self::Output, Error> {
        let config = Config::load(&get_config_path())?;
        let vm_config = VirtualMachine::load(
            &get_vm_config_path()
                .join(format!("{}.{}", run_virtual_machines_input.name, get_vminstance_extension()))
                .join("config.yaml")
            )?;
        
        let run = VmContext::new(
            &get_run_path(),
            &get_net_script(true),
            &get_net_script(false), &vm_config, &config,
            &get_vm_env_variable(),
        );

        run.run().await?;

        let qmp = qmp::client::Client::connect(&run.get_socket_path()).await?;
        qmp.invoke(qmp::types::InvokeCommand::set_vnc_password(&vm_config.vnc.password)).await?;

        Ok(())
    }
}

pub struct ShutdownVirtualMachinesInput {
    pub name: String,
}

#[async_trait]
impl Facade<ShutdownVirtualMachinesInput> for DefaultFacade {
    type Output = ();
    
    async fn invoke(&self, shutdown_virtual_machines_input: ShutdownVirtualMachinesInput) -> Result<Self::Output, Error> {
        let config = Config::load(&get_config_path())?;
        let vm_config = VirtualMachine::load(
            &get_vm_config_path()
                .join(format!("{}.{}", shutdown_virtual_machines_input.name, get_vminstance_extension()))
                .join("config.yaml")
            )?;
        
        let run = VmContext::new(
            &get_run_path(),
            &get_net_script(true),
            &get_net_script(false), &vm_config, &config,
            &get_vm_env_variable(),
        );
        
        let qmp = qmp::client::Client::connect(&run.get_socket_path()).await?;
        qmp.invoke(qmp::types::InvokeCommand::quit()).await?;

        Ok(())
    }
}


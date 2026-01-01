use std::collections::HashMap;

use vm_types::{cloudinit::{ChpasswdUser, CloudInit, PowerState, PresetNetworkConfig}, vm::{DriveConfig, NetworkConfig, VmLaunchRequest}};

use crate::{context::YaveContext, registry::{IPv4AddressRecord, NetworkInterfaceRecord}};

pub struct VmLaunchRequestBuilder<'ctx> {
    context: &'ctx YaveContext,
}

impl<'ctx> VmLaunchRequestBuilder<'ctx> {
    pub fn new(context: &'ctx YaveContext) -> VmLaunchRequestBuilder<'ctx> {
        VmLaunchRequestBuilder { context }
    }

    pub async fn build(&self, vm_id: &str) -> Result<VmLaunchRequest, crate::Error> {
        let registry = self.context.registry();
        let (vm_record, drives, nics, _) = registry.get_vm_full(vm_id).await?;
        let mut launch_request = VmLaunchRequest {
            id: vm_record.id,
            hostname: vm_record.hostname,
            vcpu: vm_record.vcpu,
            memory: vm_record.memory,
            ovmf: vm_record.ovmf,
            vnc: Some(vm_record.vnc_display),
            drives: vec![],
            networks: vec![],
        };
        for drive in drives {
            let drive_path = self.context.storage().path_for_vm(&drive.vm_id).join(format!("{}.img", drive.id));
            launch_request.drives.push(DriveConfig {
                id: drive.id,
                path: drive_path.to_string_lossy().to_string(),
                drive_media: drive.drive_bus,
            });
        }
        for nic in nics {
            launch_request.networks.push(NetworkConfig {
                id: nic.id.clone(),
                ifname: nic.ifname.clone(),
                mac: nic.mac_address.clone(),
                netdev_up_script: Some(self.context.netdev_scripts().up.clone()),
                netdev_down_script: Some(self.context.netdev_scripts().down.clone()),
            });
        }
        Ok(launch_request)
    }
}

pub struct CloudInitBuilder<'ctx> {
    context: &'ctx YaveContext,
}

impl<'ctx> CloudInitBuilder<'ctx> {
    pub fn new(context: &'ctx YaveContext) -> CloudInitBuilder<'ctx> {
        CloudInitBuilder { context }
    }

    fn create_network_config(&self, nics: &[NetworkInterfaceRecord], ipv4s: &[IPv4AddressRecord]) -> vm_types::cloudinit::PresetNetworkConfig {
        let mut interfaces = HashMap::new();
        for nic in nics {
            let mut addresses = vec![];
            for addr in ipv4s.iter().filter(|a| a.ifname == nic.ifname) {
                addresses.push(format!("{}/{}", addr.address, addr.netmask));
            }
            interfaces.insert(nic.id.clone(), vm_types::cloudinit::EthernetConfig {
                match_interface: vm_types::cloudinit::MatchInterface {
                    macaddress: nic.mac_address.clone(),
                },
                addresses,
            });
        }
        PresetNetworkConfig {
            version: 2,
            ethernets: interfaces,
        }
    }

    pub async fn build(&self, vm_id: &str, root_password: &str) -> Result<CloudInit, crate::Error> {
        let registry = self.context.registry();
        let (vm_record, _, nics, ipv4s) = registry.get_vm_full(vm_id).await?;
        let cloud_init = vm_types::cloudinit::UserDataCloudInit {
            hostname: vm_record.hostname,
            chpasswd: vm_types::cloudinit::Chpasswd {
                expire: false,
                users: vec![
                    ChpasswdUser {
                        name: "root".to_string(),
                        password: root_password.to_string(),
                        type_password: "text".to_string(),
                    },
                ],
            },
            ssh_pwauth: true,
            power_state: PowerState {
                delay: "now".to_string(),
                mode: "poweroff".to_string(),
                message: "The system is going down for power off NOW!".to_string(),
                timeout: 1,
                condition: "true".to_string(),
            },
            disable_root: false,
        };
        Ok(CloudInit {
            user_data: cloud_init,
            network_config: self.create_network_config(&nics, &ipv4s) 
        })
    }
}

use vm_types::vm::{DriveConfig, NetworkConfig, VmLaunchRequest};

use crate::context::YaveContext;

pub struct VmLaunchRequestBuilder<'ctx> {
    context: &'ctx YaveContext,
}

impl<'ctx> VmLaunchRequestBuilder<'ctx> {
    pub fn new(context: &'ctx YaveContext) -> VmLaunchRequestBuilder<'ctx> {
        VmLaunchRequestBuilder { context }
    }

    pub async fn build(&self, vm_id: &str) -> Result<VmLaunchRequest, crate::Error> {
        let registry = self.context.registry();
        let (vm_record, drives, nics) = registry.get_all_about_vm(vm_id).await?;
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

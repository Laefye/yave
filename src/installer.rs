use std::path::PathBuf;

use tempfile::tempdir;
use vm_types::{Drive, DriveDevice, IdeDevice, cloudinit::CloudConfig};

use crate::{Error, contexts::vm::VirtualMachineContext, vmrunner, tools::GenIsoImage};

pub struct Installer {
    vm: VirtualMachineContext,
    cloud_config: CloudConfig,
}

impl Installer {
    pub fn new(vm: VirtualMachineContext, cloud_config: CloudConfig) -> Self {
        Self {
            vm,
            cloud_config,
        }
    }

    async fn create_iso_image(&self, source_iso_dir: &tempfile::TempDir, output_iso: &PathBuf) -> Result<(), Error> {
        std::fs::write(source_iso_dir.path().join("user-data"), self.cloud_config.to_yaml()?)?;
        std::fs::write(source_iso_dir.path().join("meta-data"), "")?;
        std::fs::write(source_iso_dir.path().join("network-config"), "")?;

        GenIsoImage::new(&self.vm.yave_context().config().await?.cli.genisoimage)
            .create(
                source_iso_dir.path(),
                output_iso,
                "cidata"
            ).await?;
        Ok(())
    }


    pub async fn install(&self) -> Result<(), Error> {
        let source_iso_dir = tempdir()?;
        let output_iso_dir = tempdir()?;
        
        let cloudimg_path = output_iso_dir.path().join("cloudinit.iso");
        
        self.create_iso_image(&source_iso_dir, &cloudimg_path).await?;

        let mut vm = self.vm.vm_config()?.clone();
        vm.drives.insert(
            "cloudinit".to_string(),
            Drive {
                device: DriveDevice::Ide(IdeDevice {
                    boot_index: Some(vm.drives.len() as u32 + 1),
                    media_type: vm_types::MediaType::Cdrom,
                }),
                path: cloudimg_path.to_string_lossy().to_string(),
            },
        );
        
        let vmrunner = vmrunner::VmRunner::new(&self.vm).with_vm(vm);
        vmrunner.run().await?;

        let mut qmp = self.vm.connect_qmp().await?;
        qmp.on_close().await?;
        Ok(())
    }
}

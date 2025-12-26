use std::path::PathBuf;

use tempfile::tempdir;
use vm_types::{Config, Drive, DriveDevice, VirtioBlkDevice, VirtualMachine, cloudinit::CloudConfig};

use crate::{Error, tools::GenIsoImage, vmrunner::VmRunner, yavecontext::YaveContextParams};

pub struct PresetInstaller {
    input_drive: PathBuf,
    base_vm: VirtualMachine,
    cloud_init_config: CloudConfig,
}

impl PresetInstaller {
    pub fn new<P: AsRef<std::path::Path>>(base_vm: VirtualMachine, input_drive: P, cloud_init_config: CloudConfig) -> Self {
        Self {
            input_drive: input_drive.as_ref().to_path_buf(),
            base_vm,
            cloud_init_config: cloud_init_config,
        }
    }

    fn create_vm(&self, cloudimg: PathBuf) -> VirtualMachine {
        let mut vm = self.base_vm.clone();
        vm.drives.clear();
        vm.drives.insert(
            "root".to_string(),
            Drive {
                device: DriveDevice::VirtioBlk(VirtioBlkDevice {
                    boot_index: Some(1)
                }),
                path: self.input_drive.to_string_lossy().to_string(),
            },
        );
        vm.drives.insert(
            "cloudimg".to_string(),
            Drive {
                device: DriveDevice::VirtioBlk(VirtioBlkDevice {
                    boot_index: None
                }),
                path: cloudimg.to_string_lossy().to_string(),
            });
        vm
    }

    pub async fn create_iso_image(&self, source_iso_dir: &tempfile::TempDir, output_iso: &PathBuf, config: &Config) -> Result<(), Error> {
        std::fs::write(source_iso_dir.path().join("user-data"), self.cloud_init_config.to_yaml()?)?;
        std::fs::write(source_iso_dir.path().join("meta-data"), "")?;
        std::fs::write(source_iso_dir.path().join("network-config"), "")?;

        GenIsoImage::new(&config.cli.genisoimage)
            .create(
                source_iso_dir.path(),
                output_iso,
                "cidata"
            ).await?;
        Ok(())
    }


    pub async fn install(&self, config: &Config, params: &YaveContextParams) -> Result<(), Error> {
        let source_iso_dir = tempdir()?;
        let output_iso_dir = tempdir()?;
        
        let cloudimg_path = output_iso_dir.path().join(&params.cloud_init_iso_name);
        self.create_iso_image(&source_iso_dir, &cloudimg_path, config).await?;

        let vm = self.create_vm(cloudimg_path);
        let vm_runner = VmRunner::new(config.clone(), vm.clone());
        vm_runner.run(params).await?;

        let mut qmp = VmRunner::create_qmp(&vm, params).await?;
        qmp.on_close().await?;
        Ok(())
    }
}

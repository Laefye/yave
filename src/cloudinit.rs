// pub struct Installer {
//     vm: VirtualMachineContext,
//     cloud_config: CloudConfig,
// }

use std::path::Path;

use tokio::process::Command;
use vm_types::{cloudinit::CloudInit, vm::{DriveBus, DriveConfig, VmLaunchRequest}};

use crate::context::YaveContext;

struct IsoCreator {
    pub genisoimage_path: String,
}

impl IsoCreator {
    pub fn new(genisoimage_path: &str) -> Self {
        Self {
            genisoimage_path: genisoimage_path.to_string(),
        }
    }

    pub async fn create(&self, source_dir: &Path, output_iso: &Path, volume_name: &str) -> Result<(), crate::Error> {
        Command::new(&self.genisoimage_path)
            .arg("-output")
            .arg(output_iso)
            .arg("-volid")
            .arg(volume_name)
            .arg("-joliet")
            .arg("-rock")
            .arg(source_dir)
            .status()
            .await?;
        Ok(())
    }
}

struct CloudConfigIso {
    source_temp_dir: tempfile::TempDir,
    output_iso_dir: tempfile::TempDir,
}

impl CloudConfigIso {
    pub fn new() -> Result<Self, crate::Error> {
        let source_temp_dir = tempfile::tempdir()?;
        let output_iso_dir = tempfile::tempdir()?;
        Ok(Self {
            source_temp_dir,
            output_iso_dir,
        })
    }

    pub fn write_cloud_config(&self, cloud_config: &CloudInit) -> Result<(), crate::Error> {
        std::fs::write(
            self.source_temp_dir.path().join("user-data"),
            cloud_config.user_data.to_yaml()?,
        )?;
        std::fs::write(self.source_temp_dir.path().join("meta-data"), "")?;
        std::fs::write(
            self.source_temp_dir.path().join("network-config"),
                cloud_config.network_config.to_yaml()?,
        )?;
        Ok(())
    }

    pub fn output_iso_path(&self) -> std::path::PathBuf {
        self.output_iso_dir.path().join("cloudinit.iso")
    }
}

pub struct CloudInitInstaller<'ctx> {
    yave_context: &'ctx YaveContext,
    iso_creator: IsoCreator,
}

impl<'ctx> CloudInitInstaller<'ctx> {
    pub fn new(yave_context: &'ctx YaveContext) -> Self {
        let genisoimage_path = &yave_context.config().cli.genisoimage;
        let iso_creator = IsoCreator::new(genisoimage_path);
        Self {
            yave_context,
            iso_creator,
        }
    }

    async fn create_iso_image(
        &self,
        cloud_config: &CloudInit,
    ) -> Result<CloudConfigIso, crate::Error> {
        let cloudiso = CloudConfigIso::new()?;
        cloudiso.write_cloud_config(cloud_config)?;
        self.iso_creator
            .create(
                cloudiso.source_temp_dir.path(),
                &cloudiso.output_iso_path(),
                "cidata",
            )
            .await?;
        Ok(cloudiso)
    }

    pub async fn install(
        &self,
        launch_request: &VmLaunchRequest,
        cloud_config: &CloudInit,
    ) -> Result<(), crate::Error> {
        let iso = self.create_iso_image(cloud_config).await?;
        let mut launch_request = launch_request.clone();
        #[cfg(not(debug_assertions))]
        {
            launch_request.vnc = None;
        }
        launch_request.drives.push(DriveConfig {
            id: "cloudinit".to_string(),
            drive_media: DriveBus::Ide {
                media_type: vm_types::vm::DiskMediaKind::Cdrom,
                boot_index: Some(launch_request.drives.len() as u32 + 1),
            },
            path: iso.output_iso_path().to_string_lossy().to_string(),
        });
        let runtime = self.yave_context.runtime();
        runtime.run_vm(&launch_request).await?;
        let mut qmp = runtime.qmp_connect(&launch_request).await?;
        #[cfg(debug_assertions)]
        {
            println!("Cloud Init ISO {:?}", iso.output_iso_path().to_string_lossy());
            use qmp::types::InvokeCommand;

            qmp.invoke(InvokeCommand::set_vnc_password("12345678")).await?;
        }
        qmp.on_close().await?;
        Ok(())
    }
}

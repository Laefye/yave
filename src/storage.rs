use std::path::PathBuf;

use qemu::Img;

pub struct VmStorage {
    base: PathBuf,
    qemu_img: PathBuf,
}

pub enum DriveInstallMode {
    New {
        id: String,
        size: u64,
    },
    Existing {
        id: String,
        resize: u64,
        image: String,
    },
}

pub struct InstallOptions {
    pub drives: Vec<DriveInstallMode>,
}

impl VmStorage {
    pub fn new(base: impl AsRef<std::path::Path>, qemu_img: impl AsRef<std::path::Path>) -> Self {
        Self {
            base: base.as_ref().to_path_buf(),
            qemu_img: qemu_img.as_ref().to_path_buf(),
        }
    }

    pub fn path_for_vm(&self, vm_id: &str) -> std::path::PathBuf {
        self.base.join(format!("{}.vm", vm_id))
    }

    pub fn ensure_storage_dir(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.base)
    }

    async fn create_drive_image(&self, path: &PathBuf, size: u64) -> Result<(), crate::Error> {
        let args = Img::new(&self.qemu_img.to_string_lossy())
            .create(qemu::base::ImgFormat::Raw, &path.to_string_lossy(), size)
            .build();
        let mut command = tokio::process::Command::new(&args[0]);
        command.args(&args[1..]);
        command.status().await?;
        Ok(())
    }

    async fn resize_drive_image(&self, path: &PathBuf, size: u64) -> Result<(), crate::Error> {
        let args = Img::new(&self.qemu_img.to_string_lossy())
            .resize(&path.to_string_lossy(), size)
            .build();
        let mut command = tokio::process::Command::new(&args[0]);
        command.args(&args[1..]);
        command.status().await?;
        Ok(())
    }

    fn get_image_path(&self, image: &str) -> PathBuf {
        self.base.join(image).with_added_extension("img")
    }

    pub async fn install_vm(&self, vm_id: &str, options: &InstallOptions) -> Result<(), crate::Error> {
        let vm_path = self.path_for_vm(vm_id);
        std::fs::create_dir_all(&vm_path)?;
        for drive in options.drives.iter() {
            match drive {
                DriveInstallMode::New { id, size } => {
                    let drive_path = vm_path.join(id).with_added_extension("img");
                    self.create_drive_image(&drive_path, *size).await?;
                    log::debug!("Created new drive image at {:?}", drive_path);
                }
                DriveInstallMode::Existing { id, resize, image } => {
                    let drive_path = vm_path.join(id).with_added_extension("img");
                    std::fs::copy(self.get_image_path(image), &drive_path)?;
                    log::debug!("Copied existing drive image to {:?}", drive_path);
                    if *resize > 0 {
                        self.resize_drive_image(&drive_path, *resize).await?;
                        log::debug!("Resized drive image at {:?} to {}", drive_path, resize);
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn delete_vm(&self, vm_id: &str) -> Result<(), crate::Error> {
        let vm_path = self.path_for_vm(vm_id);
        if vm_path.exists() {
            tokio::fs::remove_dir_all(vm_path).await?;
        }
        Ok(())
    }
}


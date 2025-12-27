use std::path::Path;

use qemu::Img;
use tokio::process::Command;

pub struct QemuImg {
    img_tool: String,
}

impl QemuImg {
    pub fn new<P: AsRef<Path>>(p: P) -> QemuImg {
        if !p.as_ref().is_absolute() {
            panic!("Need absolute path")
        }
        Self {
            img_tool: p.as_ref().to_string_lossy().to_string(),
        }
    }

    fn create_command<P: AsRef<Path>>(&self, size: u32, path: P) -> Vec<String> {
        if !path.as_ref().is_absolute() {
            panic!("Need absolute path")
        }
        Img::new(&self.img_tool)
            .create(qemu::base::ImgFormat::Raw, &path.as_ref().to_string_lossy().to_string(), size)
            .build()
    }

    fn convert_command<P: AsRef<Path>, Q: AsRef<Path>>(&self, path: P, dest: Q) -> Vec<String> {
        if !path.as_ref().is_absolute() {
            panic!("Need absolute path")
        }
        Img::new(&self.img_tool)
            .convert(qemu::base::ImgFormat::Raw, &path.as_ref().to_string_lossy().to_string(), &dest.as_ref().to_string_lossy().to_string())
            .build()
    }

    fn resize_command<P: AsRef<Path>>(&self, size: u32, path: P) -> Vec<String> {
        if !path.as_ref().is_absolute() {
            panic!("Need absolute path")
        }
        Img::new(&self.img_tool)
            .resize(&path.as_ref().to_string_lossy().to_string(), size)
            .build()
    }

    pub async fn create<P: AsRef<Path>>(&self, size: u32, path: P) -> Result<(), std::io::Error> {
        let args = self.create_command(size, path);
        let mut command = Command::new(&args[0]);
        command.args(&args[1..]);
        command.status().await?;
        Ok(())
    }

    pub async fn convert<P: AsRef<Path>, Q: AsRef<Path>>(&self, path: P, dest: Q) -> Result<(), std::io::Error> {
        let args = self.convert_command(path, dest);
        let mut command = Command::new(&args[0]);
        command.args(&args[1..]);
        command.status().await?;
        Ok(())
    }

    pub async fn resize<P: AsRef<Path>>(&self, size: u32, path: P) -> Result<(), std::io::Error> {
        let args = self.resize_command(size, path);
        let mut command = Command::new(&args[0]);
        command.args(&args[1..]);
        command.status().await?;
        Ok(())
    }
}

pub struct GenIsoImage {
    img_tool: String,
}

impl GenIsoImage {
    pub fn new<P: AsRef<Path>>(p: P) -> GenIsoImage {
        if !p.as_ref().is_absolute() {
            panic!("Need absolute path")
        }
        Self {
            img_tool: p.as_ref().to_string_lossy().to_string(),
        }
    }

    fn create_command<P: AsRef<Path>, Q: AsRef<Path>>(&self, src: P, dest: Q, volume: &str) -> Vec<String> {
        if !src.as_ref().is_absolute() || !dest.as_ref().is_absolute() {
            panic!("Need absolute path")
        }
        vec![
            self.img_tool.clone(),
            "-output".to_string(),
            dest.as_ref().to_string_lossy().to_string(),
            "-rational-rock".to_string(),
            "-joliet".to_string(),
            "-volid".to_string(),
            volume.to_string(),
            src.as_ref().to_string_lossy().to_string(),
        ]
    }

    pub async fn create<P: AsRef<Path>, Q: AsRef<Path>>(&self, src: P, dest: Q, volume: &str) -> Result<(), std::io::Error> {
        let args = self.create_command(src, dest, volume);
        let mut command = Command::new(&args[0]);
        command.args(&args[1..]);
        command.status().await?;
        Ok(())
    }
}


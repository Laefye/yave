use std::path::Path;

use qemu::Img;
use tokio::process::Command;

pub struct Images {
    img_tool: String,
}

impl Images {
    pub fn new<P: AsRef<Path>>(p: P) -> Images {
        if !p.as_ref().is_absolute() {
            panic!("Need absolute path")
        }
        Self {
            img_tool: p.as_ref().to_string_lossy().to_string(),
        }
    }

    async fn create<P: AsRef<Path>>(&self, size: u32, path: P) -> Vec<String> {
        if !path.as_ref().is_absolute() {
            panic!("Need absolute path")
        }
        Img::new(&self.img_tool)
            .create(qemu::base::ImgFormat::Qcow2, &path.as_ref().to_string_lossy().to_string(), size)
            .build()
    }

    pub async fn run<P: AsRef<Path>>(&self, size: u32, path: P) -> Result<(), std::io::Error> {
        let args = self.create(size, path).await;
        let mut command = Command::new(&args[0]);
        command.args(&args[1..]);
        command.status().await?;
        Ok(())
    }
}

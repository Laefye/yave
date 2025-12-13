use crate::qemu::QEMU;

pub enum IdeType {
    Disk,
    Cdrom,
}

impl QEMU {
    pub fn ide_device(self, drive_id: &str, boot_index: Option<u32>, ide_type: IdeType) -> Self {
        let device_type = match ide_type {
            IdeType::Disk => "ide-hd",
            IdeType::Cdrom => "ide-cd",
        };
        let boot_arg = if let Some(index) = boot_index {
            format!(",bootindex={}", index)
        } else {
            "".to_string()
        };
        let drive_arg = format!(",drive={}", drive_id);
        
        self
            .arg("-device")
            .arg(&format!("{}{}{}", device_type, drive_arg, boot_arg))
    }
}
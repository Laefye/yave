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

    pub fn nvme_device(self, drive_id: &str, boot_index: Option<u32>, serial: &str) -> Self {
        let boot_arg = if let Some(index) = boot_index {
            format!(",bootindex={}", index)
        } else {
            "".to_string()
        };
        let drive_arg = format!(",drive={}", drive_id);
        let serial_arg = format!(",serial={}", serial);
        
        self
            .arg("-device")
            .arg(&format!("nvme{}{}{}{}", drive_arg, drive_arg, boot_arg, serial_arg))
    }

    pub fn virtio_vga(mut self) -> Self {
        self.args.push("-device".to_string());
        self.args.push("virtio-vga".to_string());
        self
    }

    pub fn netdev_tap(mut self, id: &str, ifname: &str, script: Option<&str>, downscript: Option<&str>) -> Self {
        self.args.push("-netdev".to_string());
        self.args.push(format!("tap,id={},ifname={},script={},downscript={}", id, ifname, script.unwrap_or("no"), downscript.unwrap_or("no")));
        self
    }

    pub fn network_device(mut self, netdev_id: &str, mac: &str) -> Self {
        self.args.push("-device".to_string());
        self.args.push(format!("e1000,netdev={},mac={}", netdev_id, mac));
        self
    }
}
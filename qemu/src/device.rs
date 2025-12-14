use std::path::Path;

use crate::QEMU;

pub enum MediaType {
    Disk,
    Cdrom,
}

impl QEMU {
    pub fn ide_device(self, drive_id: &str, boot_index: Option<u32>, media_type: MediaType, bus: Option<&str>) -> Self {
        let device_type = match media_type {
            MediaType::Disk => "ide-hd",
            MediaType::Cdrom => "ide-cd",
        };
        let boot_arg = if let Some(index) = boot_index {
            format!(",bootindex={}", index)
        } else {
            "".to_string()
        };
        let drive_arg = format!(",drive={}", drive_id);
        let bus_arg = if let Some(bus_name) = bus {
            format!(",bus={}", bus_name)
        } else {
            "".to_string()
        };
        self
            .arg("-device")
            .arg(&format!("{}{}{}{}", device_type, drive_arg, boot_arg, bus_arg))
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

    pub fn achi9_controller(mut self, id: &str) -> Self {
        self.args.push("-device".to_string());
        self.args.push(format!("ahci,id={}", id));
        self
    }

    pub fn virtio_vga(mut self) -> Self {
        self.args.push("-device".to_string());
        self.args.push("virtio-vga".to_string());
        self
    }

    pub fn netdev_tap<T: AsRef<Path>, S: AsRef<Path>>(mut self, id: &str, ifname: &str, script: Option<T>, downscript: Option<S>) -> Self {
        let script = match script {
            Some(s) => s.as_ref().to_string_lossy().to_string(),
            None => "no".to_string(),
        };
        let downscript = match downscript {
            Some(d) => d.as_ref().to_string_lossy().to_string(),
            None => "no".to_string(),
        };
        self.args.push("-netdev".to_string());
        self.args.push(format!("tap,id={},ifname={},script={},downscript={}", id, ifname, script, downscript));
        self
    }

    pub fn network_device(mut self, netdev_id: &str, mac: &str) -> Self {
        self.args.push("-device".to_string());
        self.args.push(format!("e1000,netdev={},mac={}", netdev_id, mac));
        self
    }
}
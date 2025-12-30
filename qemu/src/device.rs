use std::path::Path;

use vm_types::{MediaType, vm::{DiskMediaKind, DriveBus}};

use crate::KVM;

struct ArgValue {
    parts: Vec<String>,
}

impl ArgValue {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    pub fn arg<T: ToString>(mut self, value: T) -> Self {
        self.parts.push(value.to_string());
        self
    }

    pub fn key_value<T: ToString, U: ToString>(mut self, key: T, value: U) -> Self {
        self.parts.push(format!("{}={}", key.to_string(), value.to_string()));
        self
    }

    pub fn key_value_opt<T: ToString, U: ToString>(mut self, key: T, value: Option<U>) -> Self {
        if let Some(v) = value {
            self.parts.push(format!("{}={}", key.to_string(), v.to_string()));
        }
        self
    }

    pub fn build(self) -> String {
        self.parts.join(",")
    }
}

impl KVM {
    pub fn ide_device(self, drive_id: &str, boot_index: Option<u32>, media_type: &DiskMediaKind) -> Self {
        let device_type = match media_type {
            DiskMediaKind::Disk => "ide-hd",
            DiskMediaKind::Cdrom => "ide-cd",
        };
        self
            .arg("-device")
            .arg(&ArgValue::new()
                .arg(device_type)
                .key_value("drive", drive_id)
                .key_value_opt("bootindex", boot_index).build()
            )
    }

    pub fn virtio_blk(self, drive_id: &str, boot_index: Option<u32>) -> Self {
        self
            .arg("-device")
            .arg(&ArgValue::new()
                .arg("virtio-blk-pci")
                .key_value("drive", drive_id)
                .key_value_opt("bootindex", boot_index).build()
            )
    }

    pub fn virtio_vga(self) -> Self {
        self
            .arg("-device")
            .arg("virtio-vga")
    }

    pub fn netdev_tap<T: AsRef<Path>, S: AsRef<Path>>(self, id: &str, script: Option<T>, downscript: Option<S>, ifname: &str) -> Self {
        let script = match script {
            Some(s) => s.as_ref().to_string_lossy().to_string(),
            None => "no".to_string(),
        };
        let downscript = match downscript {
            Some(d) => d.as_ref().to_string_lossy().to_string(),
            None => "no".to_string(),
        };
        self
            .arg("-netdev")
            .arg(&ArgValue::new()
                .arg("tap")
                .key_value("ifname", ifname)
                .key_value("id", id)
                .key_value("script", script)
                .key_value("downscript", downscript)
                .build()
            )
    }

    pub fn network_device(self, netdev_id: &str, mac: &str) -> Self {
        self
            .arg("-device")
            .arg(&ArgValue::new()
                .arg("e1000")
                .key_value("netdev", netdev_id)
                .key_value("mac", mac)
                .build()
            )
    }
}
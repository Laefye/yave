use std::path::Path;

use crate::QEMU;


impl QEMU {
    pub fn drive<P: AsRef<Path>>(self, id: &str, filename: P) -> Self {
        self
            .arg("-drive")
            .arg(&format!("file={},if=none,id={}", filename.as_ref().to_string_lossy(), id))
    }
}
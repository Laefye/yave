use std::path::{self, Path};

use crate::qemu::{
    QEMU,
    Error,
};

#[derive(Debug, thiserror::Error)]
pub enum ArgumentError {
    #[error("Failed to convert path to absolute: {0}")]
    PathConversion(String),
}

pub fn to_absolute_path(path: &Path) -> Result<String, Error> {
    let abs_path = path::absolute(path).map_err(
        |e| ArgumentError::PathConversion(e.to_string())
    )?;
    Ok(abs_path.to_string_lossy().to_string())
}

impl QEMU {
    pub fn memory(mut self, megabytes: u32) -> Self {
        self.args.push("-m".to_string());
        self.args.push(format!("{}M", megabytes));
        self
    }

    pub fn smp(mut self, cores: u32) -> Self {
        self.args.push("-smp".to_string());
        self.args.push(cores.to_string());
        self
    }

    pub fn qmp<P: AsRef<Path>>(mut self, unix: P) -> Result<Self, Error> {
        self.args.push("-qmp".to_string());
        self.args.push(format!("unix:{},server=on,wait=off", to_absolute_path(unix.as_ref())?));
        Ok(self)
    }

    pub fn pidfile<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Error> {
        self.args.push("-pidfile".to_string());
        self.args.push(to_absolute_path(path.as_ref())?);
        Ok(self)
    }

    pub fn daemonize(mut self) -> Self {
        self.args.push("-daemonize".to_string());
        self
    }

    pub fn vnc(mut self, display: &str, password: bool) -> Self {
        self.args.push("-vnc".to_string());
        let mut vnc_arg = display.to_string();
        if password {
            vnc_arg.push_str(",password=on");
        }
        self.args.push(vnc_arg);
        self
    }
}
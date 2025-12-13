use std::path::Path;

use crate::qemu::{
    QEMU,
};

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

    pub fn qmp<P: AsRef<Path>>(mut self, unix: P) -> Self {
        self.args.push("-qmp".to_string());
        self.args.push(format!("unix:{},server=on,wait=off", unix.as_ref().to_string_lossy()));
        self
    }

    pub fn pidfile<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.args.push("-pidfile".to_string());
        self.args.push(path.as_ref().to_string_lossy().to_string());
        self
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

    pub fn name(mut self, name: &str) -> Self {
        self.args.push("-name".to_string());
        self.args.push(name.to_string());
        self
    }
}
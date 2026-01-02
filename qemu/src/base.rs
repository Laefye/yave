use std::path::Path;

use crate::{Img, KVM};

impl KVM {
    pub fn enable_kvm(mut self) -> Self {
        self.args.push("-enable-kvm".to_string());
        self
    }

    pub fn memory(mut self, megabytes: u32) -> Self {
        self.args.push("-m".to_string());
        self.args.push(format!("{}M", megabytes));
        self
    }

    pub fn smp(mut self, cores: u32) -> Self {
        self.args.push("-cpu".to_string());
        self.args.push("host".to_string());
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

    pub fn nodefaults(mut self) -> Self {
        self.args.push("-nodefaults".to_string());
        self
    }

    pub fn name(mut self, name: &str) -> Self {
        self.args.push("-name".to_string());
        self.args.push(name.to_string());
        self
    }
}

pub enum ImgFormat {
    Qcow2,
    Raw,
}

impl Img {
    pub fn create(self, format: ImgFormat, path: &str, size: u64) -> Self {
        self.arg("create")
            .arg("-f")
            .arg(match format {
                ImgFormat::Qcow2 => "qcow2",
                ImgFormat::Raw => "raw",
            })
            .arg(path)
            .arg(&format!("{}M", size))
    }

    pub fn convert(self, format: ImgFormat, src: &str, dest: &str) -> Self {
        self.arg("convert")
            .arg("-O")
            .arg(match format {
                ImgFormat::Qcow2 => "qcow2",
                ImgFormat::Raw => "raw",
            })
            .arg(src)
            .arg(dest)
    }

    pub fn resize(self, path: &str, size: u64) -> Self {
        self.arg("resize")
            .arg(path)
            .arg(&format!("{}M", size))
    }
}

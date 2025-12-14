pub mod base;
pub mod device;
pub mod drive;
pub mod ovmf;

pub struct QEMU {
    args: Vec<String>,
}

impl QEMU {
    pub fn new(binary: &str) -> Self {
        QEMU { args: vec![binary.to_string()] }
    }

    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn build(self) -> Vec<String> {
        self.args
    }
}

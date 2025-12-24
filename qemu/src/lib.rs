pub mod base;
pub mod device;
pub mod drive;
pub mod ovmf;

pub struct KVM {
    args: Vec<String>,
}

impl KVM {
    pub fn new(binary: &str) -> Self {
        KVM { args: vec![binary.to_string()] }
    }

    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn build(self) -> Vec<String> {
        self.args
    }
}

pub struct Img {
    args: Vec<String>,
}

impl Img {
    pub fn new(binary: &str) -> Self {
        Img { args: vec![binary.to_string()] }
    }

    pub fn arg(mut self, arg: &str) -> Self {
        self.args.push(arg.to_string());
        self
    }

    pub fn build(self) -> Vec<String> {
        self.args
    }
}

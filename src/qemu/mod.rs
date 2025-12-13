pub mod arguments;
pub struct QEMU {
    args: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid QEMU argument: {0}")]
    InvalidArgument(#[from] arguments::ArgumentError),
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

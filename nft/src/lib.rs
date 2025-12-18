pub mod builders;

pub struct Nft {
    args: Vec<String>
}

impl Nft {
    pub fn new(bin: &str) -> Nft {
        Nft {
            args: vec![bin.to_string()],
        }
    }

    pub fn build(self) -> Vec<String> {
        self.args
    }
}

impl Default for Nft {
    fn default() -> Self {
        Self::new("/usr/sbin/nft")
    }
}

use crate::Nft;

pub enum Family {
    Inet,
}

impl ToString for Family {
    fn to_string(&self) -> String {
        match self {
            Family::Inet => "inet".to_string(),
        }
    }
}

pub struct Add {
    nft: Nft,
}

impl Nft {
    pub fn add(mut self) -> Add {
        self.args.push("add".to_string());
        Add{ nft: self }
    }
}

impl Add {
    pub fn table(mut self, family: Family, name: &str) -> Nft {
        self.nft.args.push("table".to_string());
        self.nft.args.push(family.to_string());
        self.nft.args.push(name.to_string());
        self.nft
    }
}

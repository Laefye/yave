use md5::{Digest, Md5};

pub fn get_mac(name: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(name.as_bytes());
    let hash = hasher.finalize();
    format!(
        "52:54:{:02x}:{:02x}:{:02x}:{:02x}",
        hash[0], hash[1], hash[2], hash[3]
    )
}
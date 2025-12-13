use std::path::PathBuf;


#[cfg(debug_assertions)]
pub fn get_config_path() -> PathBuf {
    PathBuf::from("debug/config.toml")
}

#[cfg(debug_assertions)]
pub fn get_run_path() -> PathBuf {
    PathBuf::from("debug/run")
}
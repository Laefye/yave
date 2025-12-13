use std::path::PathBuf;


#[cfg(debug_assertions)]
pub fn get_config_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or(PathBuf::from("."))
        .join("debug")
        .join("config.toml")
}

#[cfg(debug_assertions)]
pub fn get_run_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or(PathBuf::from("."))
        .join("debug")
        .join("run")
}
use std::path::PathBuf;


#[cfg(debug_assertions)]
pub fn get_config_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or(PathBuf::from("."))
        .join("debug")
        .join("config.yaml")
}

#[cfg(debug_assertions)]
pub fn get_run_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or(PathBuf::from("."))
        .join("debug")
        .join("run")
}

pub fn get_net_script(up: bool) -> PathBuf {
    if up {
        std::env::current_dir()
            .unwrap_or(PathBuf::from("."))
            .join("netdevup")
    } else {
        std::env::current_dir()
            .unwrap_or(PathBuf::from("."))
            .join("netdevdown")
    }
}

pub fn get_vm_env_variable() -> String {
    "YAVE_NAME".to_string()
}

#[cfg(debug_assertions)]
pub fn get_vm_config_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or(PathBuf::from("."))
        .join("debug")
}

pub fn get_vminstance_extension() -> String {
    "vminstance".to_string()
}


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

pub fn get_vm_env_variable_path() -> String {
    "YAVE_VM_PATH".to_string()
}

use std::path::PathBuf;
#[cfg(not(debug_assertions))]
use std::path::Path;


#[cfg(debug_assertions)]
pub fn get_config_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or(PathBuf::from("."))
        .join("debug")
        .join("config.yaml")
}

#[cfg(not(debug_assertions))]
pub fn get_config_path() -> PathBuf {
    Path::new("/etc/yave/config.yaml").to_path_buf()
}

#[cfg(debug_assertions)]
pub fn get_run_path() -> PathBuf {
    std::env::current_dir()
        .unwrap_or(PathBuf::from("."))
        .join("debug")
        .join("run")
}

#[cfg(not(debug_assertions))]
pub fn get_run_path() -> PathBuf {
    Path::new("/run/yave").to_path_buf()
}

#[cfg(debug_assertions)]
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

#[cfg(not(debug_assertions))]
pub fn get_net_script(up: bool) -> PathBuf {
    if up {
        Path::new("/usr/lib/yave/netdevup").to_path_buf()

    } else {
        Path::new("/usr/lib/yave/netdevdown").to_path_buf()
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

#[cfg(not(debug_assertions))]
pub fn get_vm_config_path() -> PathBuf {
    Path::new("/var/lib/yave").to_path_buf()
}

pub fn get_vminstance_extension() -> String {
    "vm".to_string()
}


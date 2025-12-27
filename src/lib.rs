use crate::{constants::{get_config_path, get_net_script, get_run_path, get_vm_config_path, get_vminstance_extension}, yavecontext::{YaveContext, YaveContextParams}};

mod constants;
mod tools;
pub mod interface;
pub mod yavecontext;
pub mod vmcontext;
mod presetinstaller;
mod vmrunner;
pub mod contexts;
pub mod newvmrunner;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO: {0}")]
    IO(#[from] std::io::Error),
    #[error("Config Error: {0}")]
    Config(#[from] vm_types::Error),
    #[error("QMP Error: {0}")]
    QMP(#[from] qmp::Error),
    #[error("rtnetlink Error: {0}")]
    Rnetlink(#[from] rtnetlink::Error),
    #[error("Signal Error: {0}")]
    Signal(#[from] nix::Error),
    // Errors with logic
    #[error("VM Instance is not running: {0}")]
    VMNotRunning(String),
    #[error("VM Instance is running: {0}")]
    VMRunning(String),
    #[error("VM {0} not found")]
    VMNotFound(String)
}

impl Default for YaveContext {
    fn default() -> Self {
        Self::new(YaveContextParams {
            config_path: get_config_path(),
            storage_path: get_vm_config_path(),
            run_path: get_run_path(),
            vm_ext: get_vminstance_extension().into(),
            hd_ext: "qcow2".into(), // TODO: Make hd_format instead of hd_ext
            net_script_up: get_net_script(true),
            net_script_down: get_net_script(false),
            vm_config_name: "config.yaml".into(),
            vm_name_env_variable: "YAVE_VM_NAME".into(),
            preset_ext: "preset".into(),
            cloud_init_iso_name: "cloudinit.iso".into(),
        })
    }
}

impl Default for contexts::yave::YaveContext {
    fn default() -> Self {
        Self::new(
            get_config_path(),
            get_vm_config_path(),
            get_run_path(),
            &contexts::yave::NetdevScripts {
                up: get_net_script(true),
                down: get_net_script(false),
            },
        )
    }
}
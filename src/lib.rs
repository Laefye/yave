use crate::{constants::{get_config_path, get_net_script, get_run_path, get_vm_config_path, get_vminstance_extension}, yavecontext::{YaveContext, YaveParams}};

mod constants;
mod oldvmcontext;
mod interface;
mod images;
pub mod instances;
pub mod vms;
pub mod yavecontext;
pub mod vmcontext;

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
}

#[async_trait::async_trait]
pub trait Facade<T> {
    type Output;

    async fn invoke(&self, params: T) -> Result<Self::Output, Error>;
}

pub struct DefaultFacade;

impl Default for YaveContext {
    fn default() -> Self {
        Self::new(YaveParams {
            config_path: get_config_path(),
            storage_path: get_vm_config_path(),
            run_path: get_run_path(),
            vm_ext: get_vminstance_extension().into(),
            hd_ext: "qcow2".into(), // TODO: Make hd_format instead of hd_ext
            net_script_up: get_net_script(true),
            net_script_down: get_net_script(false),
            vm_config_name: "config.yaml".into(),
        })
    }
}

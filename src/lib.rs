mod constants;
mod vmcontext;
mod interface;
mod images;
pub mod instances;
pub mod vms;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO: {0}")]
    IO(#[from] std::io::Error),
    #[error("Config Error: {0}")]
    Config(#[from] vm_types::Error),
    #[error("QMP Error: {0}")]
    QMP(#[from] qmp::Error)
}

#[async_trait::async_trait]
pub trait Facade<T> {
    type Output;

    async fn invoke(&self, params: T) -> Result<Self::Output, Error>;
}

pub struct DefaultFacade;

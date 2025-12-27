use std::path::{Path, PathBuf};

use super::yave::YaveContext;

#[derive(Debug, Clone)]
pub struct VirtualMachineContext {
    yave_context: YaveContext,
    vm_config_path: PathBuf,
}

impl VirtualMachineContext {
    pub(super) fn new(yave_context: YaveContext, vm_config_path: impl AsRef<std::path::Path>) -> Self {
        Self {
            yave_context,
            vm_config_path: vm_config_path.as_ref().to_path_buf(),
        }
    }

    pub fn yave_context(&self) -> &YaveContext {
        &self.yave_context
    }

    pub fn vm_config(&self) -> &Path {
        &self.vm_config_path
    }
}

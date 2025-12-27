use std::path::{Path, PathBuf};

use qmp::types::InvokeCommand;
use vm_types::VirtualMachine;

use crate::{Error, vmrunner::VmRunner, yavecontext::{YaveContext, YaveContextParams}};

pub struct OldVmContext {
    params: YaveContextParams,
    vm_config_path: PathBuf,
}

impl OldVmContext {
    pub fn new<P: AsRef<Path>>(config_path: YaveContextParams, vm_config_path: P) -> Self {
        Self {
            params: config_path,
            vm_config_path: vm_config_path.as_ref().to_path_buf(),
        }
    }

    pub fn yave_context(&self) -> YaveContext {
        YaveContext::new(self.params.clone())
    }
    
    pub fn vm_config(&self) -> Result<VirtualMachine, Error> {
        Ok(VirtualMachine::load(&self.vm_config_path)?)
    }

    pub async fn run(&self) -> Result<(), Error> {
        unimplemented!()
    }

    pub async fn shutdown(&self) -> Result<(), Error> {
        let vm_config = self.vm_config()?;
        if !Self::vm_is_running(&vm_config, &self.params).await? {
            return Err(Error::VMNotRunning(vm_config.name.clone()));
        }
        let qmp = VmRunner::create_qmp(&vm_config, &self.params).await?;
        qmp.invoke(InvokeCommand::quit()).await?;
        Ok(())
    }

    pub async fn connect_qmp(&self) -> Result<qmp::client::Client, Error> {
        let vm_config = self.vm_config()?;
        if !Self::vm_is_running(&vm_config, &self.params).await? {
            return Err(Error::VMNotRunning(vm_config.name));
        }
        let qmp = VmRunner::create_qmp(&vm_config, &self.params).await?;
        Ok(qmp)
    }

    async fn vm_is_running(vm: &VirtualMachine, paths: &YaveContextParams) -> Result<bool, Error> {
        let pid_path = paths.with_vm_pid(&vm.name);
        let pid_str = match std::fs::read_to_string(pid_path) {
            Ok(s) => s,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(false),
            Err(e) => return Err(Error::IO(e)),
        };
        let pid: i32 = pid_str.trim().parse().unwrap_or(0);
        match nix::sys::signal::kill(nix::unistd::Pid::from_raw(pid), None) {
            Ok(_) => Ok(true),
            Err(nix::errno::Errno::ESRCH) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    pub async fn is_running(&self) -> Result<bool, Error> {
        let vm_config = self.vm_config()?;
        Self::vm_is_running(&vm_config, &self.params).await
    }
}

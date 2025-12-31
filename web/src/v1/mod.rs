use std::convert::Infallible;

use axum::{Json, Router, extract::{Path, State}, http::StatusCode, response::{IntoResponse, Sse, sse::KeepAlive}, routing::{delete, get, post}};
use axum_auth::AuthBasic;
use futures_util::{TryStreamExt};
use qmp::types::InvokeCommand;
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::ReceiverStream;
use yave::builders::VmLaunchRequestBuilder;

use crate::{AppState, auth};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/vms/", get(get_vms))
        .route("/vms/{vm}", get(get_vm))
        .route("/vms/{vm}/run", post(run_vm))
        .route("/vms/{vm}/run", delete(shutdown_vm))
        .route("/vms/{vm}/run", get(get_run_vm))
        .route("/vms/", post(create_vm))
        .route("/vms/{vm}/install", post(install_vm))
}   

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Auth error {0}")]
    Auth(#[from] auth::Error),
    #[error("Yave error {0}")]
    Yave(#[from] yave::Error),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemDetails {
    detail: String,
    status: u16,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let problem = match self {
            Error::Auth(auth::Error::InvalidCreditinals) => ProblemDetails {
                detail: "invalid creditinals".to_string(),
                status: StatusCode::UNAUTHORIZED.as_u16(),
            },
            Error::Yave(yave::Error::VMNotFound(_)) => ProblemDetails {
                detail: "virtual machine not found".to_string(),
                status: StatusCode::NOT_FOUND.as_u16(),
            },
            Error::Yave(yave::Error::VMRunning) => ProblemDetails {
                detail: "virtual machine is already running".to_string(),
                status: StatusCode::BAD_REQUEST.as_u16(),
            },
            Error::Yave(yave::Error::VMNotRunning(_)) => ProblemDetails {
                detail: "virtual machine is not running".to_string(),
                status: StatusCode::BAD_REQUEST.as_u16(),
            },
            Error::Yave(err) => {
                println!("Yave error: {:?}", err);
                ProblemDetails {
                    detail: "idk".to_string(),
                    status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                }
            },
        };
        let mut response = Json::from(&problem).into_response();
        *response.status_mut() = StatusCode::from_u16(problem.status).expect("Impossible status code");
        response
    }
}

async fn get_vms(auth: AuthBasic, State(state): State<AppState>) -> Result<impl IntoResponse, Error> {
    auth::check(&auth, &state.context.config())?;

    let registry = state.context.registry();
    let vms = registry.get_virtual_machines().await?;

    Ok(Json::from(vms))
}

async fn get_vm(auth: AuthBasic, State(state): State<AppState>, Path(vm): Path<String>) -> Result<impl IntoResponse, Error> {
    auth::check(&auth, &state.context.config())?;

    let registry = state.context.registry();
    let (vm, _, _) = registry.get_all_about_vm(&vm).await?;
    Ok(Json::from(vm))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunVMRequest {
    vnc: String,
}

async fn run_vm(auth: AuthBasic, State(state): State<AppState>, Path(vm): Path<String>, Json(payload): Json<RunVMRequest>) -> Result<impl IntoResponse, Error> {
    auth::check(&auth, &state.context.config())?;

    let builder = VmLaunchRequestBuilder::new(&state.context);
    let launch_request = builder.build(&vm).await.expect("Error building launch request");
    let runtime = state.context.runtime();
    runtime.run_vm(&launch_request).await.expect("Error running VM");
    runtime.qmp_connect(&launch_request).await.expect("Error connecting to QMP")
        .invoke(InvokeCommand::set_vnc_password(&payload.vnc)).await.expect("Error setting VNC password");

    Ok(Json::from(RunStatus {
        is_running: true,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunStatus {
    is_running: bool,
}

async fn get_run_vm(auth: AuthBasic, State(state): State<AppState>, Path(vm): Path<String>) -> Result<Json<RunStatus>, Error> {
    auth::check(&auth, &state.context.config())?;

    let builder = VmLaunchRequestBuilder::new(&state.context);
    let launch_request = builder.build(&vm).await?;
    let runtime = state.context.runtime();
    let is_running = runtime.is_running(&launch_request).await?;
    
    Ok(Json::from(RunStatus {
        is_running,
    }))
}

async fn shutdown_vm(auth: AuthBasic, State(state): State<AppState>, Path(vm): Path<String>) -> Result<impl IntoResponse, Error> {
    auth::check(&auth, &state.context.config())?;

    let builder = VmLaunchRequestBuilder::new(&state.context);
    let launch_request = builder.build(&vm).await.expect("Error building launch request");
    let runtime = state.context.runtime();
    runtime.shutdown_vm(&launch_request).await.expect("Error running VM");

    Ok(Json::from(RunStatus {
        is_running: false,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CreateDrive {
    #[serde(rename = "empty")]
    Empty {
        size: u32,
    },
    #[serde(rename = "from")]
    From {
        size: Option<u32>,
        image: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVMRequest {
    name: String,
    memory: u32,
    vcpu: u32,
    drives: Vec<CreateDrive>,
}

async fn create_vm(auth: AuthBasic, State(state): State<AppState>, Json(payload): Json<CreateVMRequest>) -> Result<impl IntoResponse, Error> {
    auth::check(&auth, &state.context.config())?;
    
    let registry = state.context.registry();
    registry.create_tables().await?;
    
    let mut drives_spec = vec![];
    let mut install_drives = vec![];
    
    for (idx, drive) in payload.drives.iter().enumerate() {
        let drive_id = format!("drive{}", idx);
        drives_spec.push(yave::registry::CreateDrive {
            id: drive_id.clone(),
            boot_order: if idx == 0 { Some(1) } else { None },
            drive_bus: vm_types::vm::DriveBus::VirtioBlk { 
                boot_index: if idx == 0 { Some(1) } else { None } 
            },
        });
        
        match drive {
            CreateDrive::Empty { size } => {
                install_drives.push(yave::storage::DriveInstallMode::New {
                    id: drive_id,
                    size: *size,
                });
            }
            CreateDrive::From { size, image } => {
                install_drives.push(yave::storage::DriveInstallMode::Existing {
                    id: drive_id,
                    resize: size.unwrap_or(15360),
                    image: image.clone(),
                });
            }
        }
    }
    
    registry.create_vm(yave::registry::CreateVirtualMachine {
        id: payload.name.clone(),
        hostname: payload.name.clone(),
        vcpu: payload.vcpu,
        memory: payload.memory,
        ovmf: true,
        network_interfaces: vec![yave::registry::CreateNetworkInterface {
            id: "net0".to_string(),
        }],
        drives: drives_spec,
    }).await?;
    
    let storage = state.context.storage();
    storage.install_vm(
        &payload.name,
        &yave::storage::InstallOptions {
            drives: install_drives,
        }
    ).await?;
    
    let (vm, _, _) = registry.get_all_about_vm(&payload.name).await?;
    Ok(Json::from(vm))
}

#[derive(Debug, Serialize, Deserialize)]
struct InstallRequest {
    hostname: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum InstallStatus {
    #[serde(rename = "started")]
    Started,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed(ProblemDetails),
}
async fn install_vm(auth: AuthBasic, State(state): State<AppState>, Path(vm): Path<String>, Json(payload): Json<InstallRequest>) -> Result<impl IntoResponse, Error> {
    auth::check(&auth, &state.context.config())?;

    let builder = VmLaunchRequestBuilder::new(&state.context);
    let launch_request = builder.build(&vm).await?;
    let context = state.context.clone();
    let cloud_config = vm_types::cloudinit::CloudInit {
        hostname: payload.hostname,
        chpasswd: vm_types::cloudinit::Chpasswd {
            expire: false,
            users: vec![
                vm_types::cloudinit::ChpasswdUser {
                    name: "root".to_string(),
                    password: payload.password,
                    type_password: "text".to_string(),
                }
            ],
        },
        ssh_pwauth: true,
        power_state: Default::default(),
    };
    
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<InstallStatus, Infallible>>(1);
    let stream = ReceiverStream::new(rx)
        .map_ok(|status| axum::response::sse::Event::default().json_data(status).unwrap());
    
    tokio::spawn(async move {
        let installer = yave::cloudinit::CloudInitInstaller::new(&context); // Create installer inside spawn
        tx.send(Ok(InstallStatus::Started)).await.ok();
        match installer.install(&launch_request, &cloud_config).await {
            Ok(_) => {
                tx.send(Ok(InstallStatus::Completed)).await.ok();
            }
            Err(err) => {
                let problem = ProblemDetails {
                    detail: format!("installation failed: {}", err),
                    status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                };
                tx.send(Ok(InstallStatus::Failed(problem))).await.ok();
            }
        }
    });

    let sse = Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(10))
            .text("keep-alive-text"),
    );
    Ok(sse.into_response())
}


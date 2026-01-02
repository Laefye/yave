use std::convert::Infallible;

use axum::{Json, Router, extract::{Path, State}, http::StatusCode, response::{IntoResponse, Sse, sse::KeepAlive}, routing::{delete, get, post}};
use axum_auth::AuthBasic;
use futures_util::TryStreamExt;
use qmp::types::InvokeCommand;
use tokio_stream::wrappers::ReceiverStream;
use yave::builders::{CloudInitBuilder, VmLaunchRequestBuilder};

use crate::{AppState, auth};
mod types;

pub use types::{Error, ProblemDetails, RunVMRequest, RunStatus, CreateDrive, CreateVMRequest, InstallRequest, InstallStatus};

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

async fn get_vms(auth: AuthBasic, State(state): State<AppState>) -> Result<impl IntoResponse, Error> {
    auth::check(&auth, &state.context.config())?;

    let registry = state.context.registry();
    let vms = registry.get_virtual_machines().await?;

    Ok(Json::from(vms))
}

async fn get_vm(auth: AuthBasic, State(state): State<AppState>, Path(vm): Path<String>) -> Result<impl IntoResponse, Error> {
    auth::check(&auth, &state.context.config())?;

    let registry = state.context.registry();
    let (vm, _, _, _) = registry.get_vm_full(&vm).await?;
    Ok(Json::from(vm))
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
    
    let vm = registry.create_vm(yave::registry::CreateVirtualMachine {
        id: payload.id.clone(),
        hostname: payload.hostname.clone(),
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
        &payload.id,
        &yave::storage::InstallOptions {
            drives: install_drives,
        }
    ).await?;
    
    Ok(Json::from(vm))
}

async fn install_vm(auth: AuthBasic, State(state): State<AppState>, Path(vm): Path<String>, Json(payload): Json<InstallRequest>) -> Result<impl IntoResponse, Error> {
    auth::check(&auth, &state.context.config())?;

    let builder = VmLaunchRequestBuilder::new(&state.context);
    let launch_request = builder.build(&vm).await?;
    let cloud_config = CloudInitBuilder::new(&state.context)
        .build(&vm, &payload.password)
        .await?;
    let context = state.context.clone();
    
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


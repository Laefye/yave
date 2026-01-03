use std::convert::Infallible;

use axum::{
    Json, Router,
    extract::{Path, State},
    response::{Sse, sse::KeepAlive},
    routing::{delete, get, post},
};
use axum_auth::AuthBasic;
use futures_util::StreamExt;
use tokio_stream::wrappers::ReceiverStream;
use yave::builders::{CloudInitBuilder, VmLaunchRequestBuilder};

use crate::{AppState, auth, v1::types::{DriveDef, IpV4AddressInfo}};
mod types;

pub use types::{
    Error, ApiResponse, CreateVMRequest, StartVMRequest,
    InstallRequest, InstallStatus, VMInfo, NetworkInterface, 
    NetworkConfig, AddIpV4Request, VMRuntime
};

pub fn router() -> Router<AppState> {
    Router::new()
        // VMs endpoints
        .route("/vm", get(list_vms))
        .route("/vm", post(create_vm))
        .route("/vm/{vm_id}", get(get_vm_info))
        .route("/vm/{vm_id}", delete(delete_vm))
        
        // Runtime endpoints
        .route("/vm/{vm_id}/start", post(start_vm))
        .route("/vm/{vm_id}/stop", post(stop_vm))
        .route("/vm/{vm_id}/reboot", post(reboot_vm))
        .route("/vm/{vm_id}/status", get(get_vm_status))
        
        // Network endpoints
        .route("/vm/{vm_id}/network", get(get_network_config))
        .route("/vm/{vm_id}/network/interfaces/{interface_id}/ipv4", get(get_ip_address))
        .route("/vm/{vm_id}/network/interfaces/{interface_id}/ipv4", post(add_ip_address))
        .route("/vm/{vm_id}/network/interfaces/{interface_id}/ipv4", delete(remove_ip_address))
        
        // Drives endpoints
        .route("/vm/{vm_id}/drives", post(reinstall_drives))

        // Installation endpoints
        .route("/vm/{vm_id}/install", post(install_vm))
}


// ============================================================================
// VM Management Handlers
// ============================================================================

/// List all virtual machines
async fn list_vms(
    auth: AuthBasic,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<VMInfo>>>, Error> {
    auth::check(&auth, &state.context.config())?;

    let registry = state.context.registry();
    let vms = registry.get_virtual_machines().await?;

    let mut vm_infos = vec![];
    for vm in vms {
        vm_infos.push(VMInfo {
            id: vm.id,
            hostname: vm.hostname,
            memory: vm.memory,
            vcpu: vm.vcpu,
            vnc_display: vm.vnc_display,
        });
    }

    Ok(Json(ApiResponse::ok(vm_infos)))
}

/// Get virtual machine info
async fn get_vm_info(
    auth: AuthBasic,
    State(state): State<AppState>,
    Path(vm_id): Path<String>,
) -> Result<Json<ApiResponse<VMInfo>>, Error> {
    auth::check(&auth, &state.context.config())?;

    let registry = state.context.registry();
    let vm = registry.get_vm_by_id(&vm_id).await?;

    let info = VMInfo {
        id: vm.id,
        hostname: vm.hostname,
        memory: vm.memory,
        vcpu: vm.vcpu,
        vnc_display: vm.vnc_display,
    };

    Ok(Json(ApiResponse::ok(info)))
}

/// Delete virtual machine
async fn delete_vm(
    auth: AuthBasic,
    State(state): State<AppState>,
    Path(vm_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, Error> {
    auth::check(&auth, &state.context.config())?;

    let builder = VmLaunchRequestBuilder::new(&state.context);
    let launch_request = builder.build(&vm_id).await?;
    let runtime = state.context.runtime();
    if runtime.is_running(&launch_request).await? {
        runtime.shutdown_vm(&launch_request).await?;
    }
    let storage = state.context.storage();
    storage.delete_vm(&vm_id).await?;
    let registry = state.context.registry();
    registry.delete_vm(&vm_id).await?;
    Ok(Json(ApiResponse::ok(
        "VM deleted successfully".to_string(),
    )))
}

// ============================================================================
// Runtime Handlers
// ============================================================================

/// Start virtual machine
async fn start_vm(
    auth: AuthBasic,
    State(state): State<AppState>,
    Path(vm_id): Path<String>,
    Json(payload): Json<StartVMRequest>,
) -> Result<Json<ApiResponse<VMRuntime>>, Error> {
    auth::check(&auth, &state.context.config())?;

    let builder = VmLaunchRequestBuilder::new(&state.context);
    let launch_request = builder.build(&vm_id).await?;
    let runtime = state.context.runtime();

    runtime.run_vm(&launch_request).await?;

    if let Ok(client) = runtime.qmp_connect(&launch_request).await {
        if let Some(vnc_password) = &payload.vnc_password {
            let cmd = qmp::types::InvokeCommand::set_vnc_password(vnc_password);
            let _ = client.invoke(cmd).await;
        }
    }

    let status = VMRuntime {
        is_running: true,
    };

    Ok(Json(ApiResponse::ok(status)))
}

/// Stop virtual machine
async fn stop_vm(
    auth: AuthBasic,
    State(state): State<AppState>,
    Path(vm_id): Path<String>,
) -> Result<Json<ApiResponse<VMRuntime>>, Error> {
    auth::check(&auth, &state.context.config())?;

    let builder = VmLaunchRequestBuilder::new(&state.context);
    let launch_request = builder.build(&vm_id).await?;
    let runtime = state.context.runtime();

    runtime.shutdown_vm(&launch_request).await?;

    let status = VMRuntime {
        is_running: false,
    };

    Ok(Json(ApiResponse::ok(status)))
}

/// Restart virtual machine
async fn reboot_vm(
    auth: AuthBasic,
    State(state): State<AppState>,
    Path(vm_id): Path<String>,
) -> Result<Json<ApiResponse<VMRuntime>>, Error> {
    auth::check(&auth, &state.context.config())?;
    let builder = VmLaunchRequestBuilder::new(&state.context);
    let launch_request = builder.build(&vm_id).await?;
    let runtime = state.context.runtime();
    runtime.reboot_vm(&launch_request).await?;

    let status = VMRuntime {
        is_running: true,
    };

    Ok(Json(ApiResponse::ok(status)))
}

/// Get virtual machine runtime status
async fn get_vm_status(
    auth: AuthBasic,
    State(state): State<AppState>,
    Path(vm_id): Path<String>,
) -> Result<Json<ApiResponse<VMRuntime>>, Error> {
    auth::check(&auth, &state.context.config())?;

    let builder = VmLaunchRequestBuilder::new(&state.context);
    let launch_request = builder.build(&vm_id).await?;
    let runtime = state.context.runtime();
    let is_running = runtime.is_running(&launch_request).await?;

    let status = VMRuntime {
        is_running,
    };

    Ok(Json(ApiResponse::ok(status)))
}

// ============================================================================
// Network Handlers
// ============================================================================

/// Get network configuration
async fn get_network_config(
    auth: AuthBasic,
    State(state): State<AppState>,
    Path(vm_id): Path<String>,
) -> Result<Json<ApiResponse<NetworkConfig>>, Error> {
    auth::check(&auth, &state.context.config())?;

    let registry = state.context.registry();
    let nic_records = registry.get_network_interfaces_by_vm_id(&vm_id).await?;

    let interfaces = nic_records
        .into_iter()
        .map(|nic| {
            NetworkInterface {
                id: nic.id,
                ifname: nic.ifname,
                mac_address: nic.mac_address,
            }
        })
        .collect();

    let config = NetworkConfig { interfaces };
    Ok(Json(ApiResponse::ok(config)))
}

/// Add IP address to network interface
async fn add_ip_address(
    auth: AuthBasic,
    State(state): State<AppState>,
    Path((vm_id, interface_id)): Path<(String, String)>,
    Json(payload): Json<AddIpV4Request>,
) -> Result<Json<ApiResponse<NetworkInterface>>, Error> {
    auth::check(&auth, &state.context.config())?;

    // Validate IP address format
    validate_ip_address(&payload.ip_address)?;

    let registry = state.context.registry();
    let nic_records = registry.get_network_interfaces_by_vm_id(&vm_id).await?;

    let nic = nic_records
        .into_iter()
        .find(|ni| ni.id == interface_id)
        .ok_or(Error::NetworkInterfaceNotFound)?;

    // Add IP address to the database
    registry.add_ipv4_address(yave::registry::AddIPv4Address {
        ifname: nic.ifname.clone(),
        address: payload.ip_address.clone(),
        netmask: payload.netmask,
        gateway: payload.gateway.clone(),
    }).await?;

    let result = NetworkInterface {
        id: interface_id,
        ifname: nic.ifname,
        mac_address: nic.mac_address,
    };

    Ok(Json(ApiResponse::ok(result)))
}

/// Remove IP address from network interface
async fn remove_ip_address(
    auth: AuthBasic,
    State(state): State<AppState>,
    Path((vm_id, interface_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<String>>, Error> {
    auth::check(&auth, &state.context.config())?;

    let registry = state.context.registry();
    let (_, _, nic_records, _) = registry.get_vm_full(&vm_id).await?;

    let _nic = nic_records
        .into_iter()
        .find(|ni| ni.id == interface_id)
        .ok_or(Error::NetworkInterfaceNotFound)?;

    println!("Removing all IP addresses from interface {}", interface_id);

    Ok(Json(ApiResponse::ok(
        "IP address removal requested successfully".to_string(),
    )))
}

// ============================================================================
// Installation Handlers
// ============================================================================

/// Install virtual machine with cloud-init
async fn install_vm(
    auth: AuthBasic,
    State(state): State<AppState>,
    Path(vm_id): Path<String>,
    Json(payload): Json<InstallRequest>,
) -> Result<Sse<impl futures_util::stream::Stream<Item = Result<axum::response::sse::Event, Infallible>>>, Error> {
    auth::check(&auth, &state.context.config())?;

    let builder = VmLaunchRequestBuilder::new(&state.context);
    let launch_request = builder.build(&vm_id).await?;
    let cloud_config = CloudInitBuilder::new(&state.context)
        .build(&vm_id, &payload.password)
        .await?;
    let context = state.context.clone();

    let (tx, rx) = tokio::sync::mpsc::channel::<ApiResponse<InstallStatus>>(1);
    
    let stream = ReceiverStream::new(rx).map(|response| {
        Ok(axum::response::sse::Event::default()
            .json_data(response)
            .unwrap())
    });

    tokio::spawn(async move {
        let installer = yave::cloudinit::CloudInitInstaller::new(&context);

        let _ = tx
            .send(ApiResponse::ok(InstallStatus::Started))
            .await;

        match installer.install(&launch_request, &cloud_config).await {
            Ok(_) => {
                let _ = tx
                    .send(ApiResponse::ok(InstallStatus::Completed))
                    .await;
            }
            Err(err) => {
                let status = InstallStatus::Failed {
                    message: format!("Installation failed: {}", err),
                };
                let _ = tx.send(ApiResponse::ok(status)).await;
            }
        }
    });

    let sse = Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(10))
            .text("keep-alive"),
    );

    Ok(sse)
}

/// Create new virtual machine
async fn create_vm(
    auth: AuthBasic,
    State(state): State<AppState>,
    Json(payload): Json<CreateVMRequest>,
) -> Result<Json<ApiResponse<VMInfo>>, Error> {
    auth::check(&auth, &state.context.config())?;

    let registry = state.context.registry();
    registry.create_tables().await?;

    let mut drives_spec = vec![];
    let mut install_drives = vec![];

    for (idx, drive) in payload.drives.iter().enumerate() {
        let drive_id = format!("drive{}", idx);
        drives_spec.push(yave::registry::CreateDrive {
            id: drive_id.clone(),
            drive_bus: vm_types::vm::DriveBus::VirtioBlk {
                boot_index: Some(idx as u32 + 1),
            },
        });

        match drive {
            DriveDef::Empty { size } => {
                install_drives.push(yave::storage::DriveInstallMode::New {
                    id: drive_id,
                    size: *size,
                });
            }
            DriveDef::From { size, image } => {
                let image = image.clone();
                install_drives.push(yave::storage::DriveInstallMode::Existing {
                    id: drive_id,
                    resize: *size,
                    image,
                });
            }
        }
    }

    let vm = registry
        .create_vm(yave::registry::CreateVirtualMachine {
            id: payload.id.clone(),
            hostname: payload.hostname.clone(),
            vcpu: payload.vcpu,
            memory: payload.memory,
            ovmf: true,
            network_interfaces: vec![yave::registry::CreateNetworkInterface {
                id: "net0".to_string(),
            }],
            drives: drives_spec,
        })
        .await?;

    let storage = state.context.storage();
    storage
        .install_vm(
            &payload.id,
            &yave::storage::InstallOptions {
                drives: install_drives,
            },
        )
        .await?;

    let info = VMInfo {
        id: vm.id,
        hostname: vm.hostname,
        memory: vm.memory,
        vcpu: vm.vcpu,
        vnc_display: vm.vnc_display,
    };

    Ok(Json(ApiResponse::ok(info)))
}

async fn get_ip_address(
    auth: AuthBasic,
    State(state): State<AppState>,
    Path((vm_id, interface_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<Vec<IpV4AddressInfo>>>, Error> {
    auth::check(&auth, &state.context.config())?;

    let registry = state.context.registry();
    let nic_records = registry.get_network_interfaces_by_vm_id(&vm_id).await?;

    let nic = nic_records
        .into_iter()
        .find(|ni| ni.id == interface_id)
        .ok_or(Error::NetworkInterfaceNotFound)?;

    let ip_addresses = registry
        .get_ipv4_by_ifname(&nic.ifname)
        .await?
        .into_iter()
        .map(|ip_record| IpV4AddressInfo {
            ip_address: ip_record.address,
            netmask: ip_record.netmask,
            gateway: ip_record.gateway,
            is_default: ip_record.is_default,
        })
        .collect();

    Ok(Json(ApiResponse::ok(ip_addresses)))
}

// ============================================================================
// Drive Handlers
// ============================================================================

async fn reinstall_drives(
    auth: AuthBasic,
    State(state): State<AppState>,
    Path(vm_id): Path<String>,
    Json(payload): Json<Vec<DriveDef>>,
) -> Result<Json<ApiResponse<()>>, Error> {
    auth::check(&auth, &state.context.config())?;

    let registry = state.context.registry();
    let vm = registry.get_vm_by_id(&vm_id).await?;

    let storage = state.context.storage();
    let mut spec_drives = vec![];
    let mut install_drives = vec![];

    for (idx, drive) in payload.iter().enumerate() {
        let drive_id = format!("drive{}", idx);

        spec_drives.push(yave::registry::CreateDrive {
            id: drive_id.clone(),
            drive_bus: vm_types::vm::DriveBus::VirtioBlk {
                boot_index: Some(idx as u32 + 1),
            },
        });

        match drive {
            DriveDef::Empty { size } => {
                install_drives.push(yave::storage::DriveInstallMode::New {
                    id: drive_id,
                    size: *size,
                });
            }
            DriveDef::From { size, image } => {
                let image = image.clone();
                install_drives.push(yave::storage::DriveInstallMode::Existing {
                    id: drive_id,
                    resize: *size,
                    image,
                });
            }
        }
    }

    registry
        .replace_drives(&vm.id, spec_drives)
        .await?;

    storage
        .install_vm(
            &vm_id,
            &yave::storage::InstallOptions {
                drives: install_drives,
            },
        )
        .await?;

    Ok(Json(ApiResponse::ok(())))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Validate IP address format
fn validate_ip_address(ip: &str) -> Result<(), Error> {
    let parts: Vec<&str> = ip.split('.').collect();

    if parts.len() != 4 {
        return Err(Error::InvalidIp(
            "IP must have 4 octets".to_string(),
        ));
    }

    for (i, part) in parts.iter().enumerate() {
        match part.parse::<u8>() {
            Ok(num) => {
                if i == 0 && (num == 0 || num > 223) {
                    return Err(Error::InvalidIp(
                        "Invalid first octet".to_string(),
                    ));
                }
            }
            Err(_) => {
                return Err(Error::InvalidIp(
                    format!("Invalid octet: {}", part),
                ))
            }
        }
    }

    Ok(())
}


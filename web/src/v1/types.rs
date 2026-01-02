use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::auth;

// ============================================================================
// Error Handling
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Authentication failed: {0}")]
    Auth(#[from] auth::AuthError),
    #[error("Virtual machine error: {0}")]
    Yave(#[from] yave::Error),
    #[error("Invalid IP address: {0}")]
    InvalidIp(String),
    #[error("Network interface not found")]
    NetworkInterfaceNotFound,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(code: String, message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code,
                message,
                details: None,
            }),
        }
    }
}

impl Error {
    /// Convert error to API response with appropriate HTTP status code
    fn to_response(&self) -> (StatusCode, String) {
        match self {
            Error::Auth(auth::AuthError::InvalidCreditinals) => (
                StatusCode::UNAUTHORIZED,
                "INVALID_CREDENTIALS".to_string(),
            ),
            Error::Yave(yave::Error::VMNotFound) => (
                StatusCode::NOT_FOUND,
                "VM_NOT_FOUND".to_string(),
            ),
            Error::Yave(yave::Error::VMRunning) => (
                StatusCode::CONFLICT,
                "VM_ALREADY_RUNNING".to_string(),
            ),
            Error::Yave(yave::Error::VMNotRunning(_)) => (
                StatusCode::BAD_REQUEST,
                "VM_NOT_RUNNING".to_string(),
            ),
            Error::InvalidIp(_) => (
                StatusCode::BAD_REQUEST,
                "INVALID_IP_ADDRESS".to_string(),
            ),
            Error::NetworkInterfaceNotFound => (
                StatusCode::NOT_FOUND,
                "NETWORK_INTERFACE_NOT_FOUND".to_string(),
            ),
            Error::Yave(err) => {
                eprintln!("Unhandled Yave error: {err:?}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "INTERNAL_ERROR".to_string(),
                )
            }
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let (status, code) = self.to_response();
        let response = ApiResponse::<()>::error(code, self.to_string());

        let mut http_response = Json(response).into_response();
        *http_response.status_mut() = status;
        http_response
    }
}

// ============================================================================
// VM Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum DriveDef {
    #[serde(rename = "empty")]
    Empty {
        size: u64,
    },
    #[serde(rename = "from")]
    From {
        size: u64,
        image: String,
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateVMRequest {
    pub id: String,
    pub hostname: String,
    pub memory: u32,
    pub vcpu: u32,
    pub drives: Vec<DriveDef>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VMInfo {
    pub id: String,
    pub hostname: String,
    pub memory: u32,
    pub vcpu: u32,
    pub vnc_display: String,
}

// ============================================================================
// Network Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkInterface {
    pub id: String,
    pub ifname: String,
    pub mac_address: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddIpRequest {
    pub ip_address: String,
    pub netmask: u32,
    pub gateway: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IpAddressInfo {
    pub ip_address: String,
    pub netmask: u32,
    pub gateway: Option<String>,
    pub is_default: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetworkConfig {
    pub interfaces: Vec<NetworkInterface>,
}

// ============================================================================
// Installation Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstallRequest {
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InstallStatus {
    #[serde(rename = "started")]
    Started,
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed { message: String },
}

// ============================================================================
// Runtime Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StartVMRequest {
    pub vnc_password: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VMRuntime {
    pub is_running: bool,
}

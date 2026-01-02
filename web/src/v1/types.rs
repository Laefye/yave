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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemDetails {
    pub detail: String,
    pub status: u16,
}

impl Error {
    /// Convert error to problem details with appropriate HTTP status code
    fn to_problem_details(&self) -> ProblemDetails {
        match self {
            Error::Auth(auth::AuthError::InvalidCreditinals) => {
                ProblemDetails {
                    detail: "Invalid credentials".to_string(),
                    status: StatusCode::UNAUTHORIZED.as_u16(),
                }
            }
            Error::Yave(yave::Error::VMNotFound) => {
                ProblemDetails {
                    detail: "Virtual machine not found".to_string(),
                    status: StatusCode::NOT_FOUND.as_u16(),
                }
            }
            Error::Yave(yave::Error::VMRunning) => {
                ProblemDetails {
                    detail: "Virtual machine is already running".to_string(),
                    status: StatusCode::BAD_REQUEST.as_u16(),
                }
            }
            Error::Yave(yave::Error::VMNotRunning(_)) => {
                ProblemDetails {
                    detail: "Virtual machine is not running".to_string(),
                    status: StatusCode::BAD_REQUEST.as_u16(),
                }
            }
            Error::Yave(err) => {
                eprintln!("Unhandled Yave error: {err:?}");
                ProblemDetails {
                    detail: "Internal server error".to_string(),
                    status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                }
            }
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let problem = self.to_problem_details();
        let status = StatusCode::from_u16(problem.status)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        
        let mut response = Json(problem).into_response();
        *response.status_mut() = status;
        response
    }
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct RunVMRequest {
    pub vnc: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunStatus {
    pub is_running: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CreateDrive {
    #[serde(rename = "empty")]
    Empty { size: u32 },
    #[serde(rename = "from")]
    From {
        size: Option<u32>,
        image: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateVMRequest {
    pub id: String,
    pub hostname: String,
    pub memory: u32,
    pub vcpu: u32,
    pub drives: Vec<CreateDrive>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InstallRequest {
    pub hostname: String,
    pub password: String,
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

use axum::{Json, Router, extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get, post}};
use axum_auth::AuthBasic;
use serde::{Deserialize, Serialize};

use crate::{AppState, auth};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/vms/", get(get_vms))
        .route("/vms/{vm}", get(get_vm))
        .route("/vms/{vm}/run", post(run_vm))
        .route("/vms/{vm}/run", get(get_run_vm))
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
            Error::Yave(yave::Error::VMRunning(_)) => ProblemDetails {
                detail: "virtual machine is already running".to_string(),
                status: StatusCode::BAD_REQUEST.as_u16(),
            },
            Error::Yave(yave::Error::VMNotRunning(_)) => ProblemDetails {
                detail: "virtual machine is not running".to_string(),
                status: StatusCode::BAD_REQUEST.as_u16(),
            },
            Error::Yave(_) => ProblemDetails {
                detail: "idk".to_string(),
                status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
            },
        };
        let mut response = Json::from(&problem).into_response();
        *response.status_mut() = StatusCode::from_u16(problem.status).expect("Impossible status code");
        response
    }
}

async fn get_vms(auth: AuthBasic, State(state): State<AppState>) -> Result<impl IntoResponse, Error> {
    auth::check(&auth, &state.context.config()?)?;

    Ok(Json::from(state.context.list()?))
}

async fn get_vm(auth: AuthBasic, State(state): State<AppState>, Path(vm): Path<String>) -> Result<impl IntoResponse, Error> {
    auth::check(&auth, &state.context.config()?)?;

    let vm = state.context.open_vm(&vm)?;
    Ok(Json::from(vm.vm_config()?))
}

async fn run_vm(auth: AuthBasic, State(state): State<AppState>, Path(vm): Path<String>) -> Result<impl IntoResponse, Error> {
    auth::check(&auth, &state.context.config()?)?;

    let vm = state.context.open_vm(&vm)?;
    vm.run().await?;
    Ok(Json::from(()))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RunStatus {
    is_running: bool,
}

async fn get_run_vm(auth: AuthBasic, State(state): State<AppState>, Path(vm): Path<String>) -> Result<impl IntoResponse, Error> {
    auth::check(&auth, &state.context.config()?)?;

    let vm = state.context.open_vm(&vm)?;
    Ok(Json::from(RunStatus {
        is_running: vm.is_running().await?,
    }))
}

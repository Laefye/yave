use axum::{
    Json, Router, extract::{Path, State}, http::StatusCode, response::IntoResponse, routing::{get, post}
};
use axum_auth::AuthBasic;
use serde::{Deserialize, Serialize};
use yave::yavecontext::YaveContext;

mod auth;
mod v1;

#[derive(Clone)]
struct AppState {
    pub context: YaveContext,
}

#[tokio::main]
async fn main() {
    let context = YaveContext::default();
    let config = context.config().expect("Failed to load config");
    let app = Router::new()
        .nest("/v1/", v1::router())
        .with_state(AppState {
            context,
        });
    let listener = tokio::net::TcpListener::bind(&config.api.listen).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


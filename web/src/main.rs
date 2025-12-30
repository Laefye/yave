use axum::Router;
use yave::contexts::yave::YaveContext;

mod auth;
mod v1;

#[derive(Clone)]
struct AppState {
    pub context: YaveContext,
}

#[tokio::main]
async fn main() {
    let context = YaveContext::default();
    let app = Router::new()
        .nest("/v1/", v1::router())
        .with_state(AppState {
            context: context.clone(),
        });
    println!("Listening on {}", context.config().api.listen);
    let listener = tokio::net::TcpListener::bind(&context.config().api.listen).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}


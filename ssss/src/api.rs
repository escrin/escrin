use std::net::{Ipv4Addr, SocketAddrV4};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
struct AppState {}

#[derive(Debug)]
struct Error(anyhow::Error);

impl<T: Into<anyhow::Error>> From<T> for Error {
    fn from(e: T) -> Self {
        Self(e.into())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        tracing::error!(error = ?self.0, "api error");
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub async fn serve(port: u16) {
    let bind_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(listener, make_router(AppState {}))
        .await
        .unwrap();
}

fn make_router(state: AppState) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/omni-key", post(get_omni_key))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

async fn root() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn get_omni_key(
    State(AppState { .. }): State<AppState>,
    Json(OmniKeyRequest { .. }): Json<OmniKeyRequest>,
) -> Result<Json<OmniKeyResponse>, Error> {
    Ok(Json(OmniKeyResponse {}))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct OmniKeyRequest {}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct OmniKeyResponse {}

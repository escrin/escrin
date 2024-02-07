use std::net::{Ipv4Addr, SocketAddrV4};

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use ethers::{
    middleware::contract::{Eip712, EthAbiType},
    types::{transaction::eip712::Eip712, Address, H256},
};
use serde::{Deserialize, Serialize};

use crate::{store::Store, types::ShareId};

#[derive(Clone)]
struct AppState<S> {
    store: S,
}

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

pub async fn serve<S: Store>(store: S, port: u16) {
    let bind_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(listener, make_router(AppState { store }))
        .await
        .unwrap();
}

fn make_router<S: Store>(state: AppState<S>) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/omni-key", post(get_omni_key))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state)
}

async fn root() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn get_omni_key<S: Store>(
    State(AppState { store }): State<AppState<S>>,
    Json(OmniKeyRequest { share, signature }): Json<OmniKeyRequest>,
) -> Result<Result<Json<OmniKeyResponse>, StatusCode>, Error> {
    let req721_hash = H256(
        OmniKeyRequest721 {
            chain: share.identity.chain,
            registry: share.identity.registry,
            identity: share.identity.id.0,
            share_version: share.version,
        }
        .encode_eip712()
        .unwrap(),
    );
    let sig: ethers::types::Signature = match signature.as_ref().try_into() {
        Ok(sig) => sig,
        Err(_) => return Ok(Err(StatusCode::BAD_REQUEST)),
    };
    let requester = match sig.recover(req721_hash) {
        Ok(a) => a,
        Err(_) => return Ok(Err(StatusCode::FORBIDDEN)),
    };
    if store.read_permit(share, requester).await?.is_none() {
        return Ok(Err(StatusCode::UNAUTHORIZED));
    }
    let share = match store.get_share(share).await? {
        Some(s) => s.to_vec(),
        None => return Ok(Err(StatusCode::NOT_FOUND)),
    };
    Ok(Ok(Json(OmniKeyResponse {
        share: share.into(),
    })))
}

type EthSignatureBytes = [u8; 65];

#[derive(Clone, Debug, Serialize, Deserialize)]
struct OmniKeyRequest {
    share: ShareId,
    #[serde(with = "hex::serde")]
    signature: EthSignatureBytes,
}

#[derive(Clone, Debug, Serialize)]
struct OmniKeyResponse {
    #[serde(with = "hex::serde")]
    share: zeroize::Zeroizing<Vec<u8>>,
}

#[derive(Clone, Default, EthAbiType, Eip712)]
#[eip712(
    name = "OmniKeyRequest",
    version = "1",
    chain_id = 0,
    verifying_contract = "0x0000000000000000000000000000000000000000"
)]
pub struct OmniKeyRequest721 {
    chain: u64,
    registry: Address,
    identity: H256,
    share_version: u64,
}

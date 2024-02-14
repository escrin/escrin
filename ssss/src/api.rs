use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddrV4},
};

use axum::{
    extract::{Path, Query, State},
    http::{
        header::{self, HeaderMap, HeaderName},
        Method, StatusCode,
    },
    response::{IntoResponse, Response},
    routing::{any, delete, get, post},
    Json, Router,
};
use ethers::{
    middleware::contract::{Eip712, EthAbiType},
    types::{transaction::eip712::Eip712, Address, H256},
};
use serde::{Deserialize, Serialize};
use tower_http::cors;

use crate::{eth, store::Store, types::*, utils::retry_times, verify};

#[derive(Clone)]
struct AppState<S> {
    store: S,
    sssss: HashMap<ChainId, eth::SsssPermitter>,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("{0}")]
    BadRequest(String),
    #[error("unable to find the requested {0}")]
    NotFound(String),
    #[error("{0}")]
    Unauthorized(String),
    #[error("{0}")]
    Forbidden(String),
    #[error("internal server error")]
    Unhandled(#[from] anyhow::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        if let Error::Unhandled(e) = &self {
            tracing::error!(error = ?e, "api error");
        }
        let status_code = match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::Unhandled(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (
            status_code,
            Json(ErrorResponse {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub async fn serve<S: Store>(store: S, sssss: impl Iterator<Item = eth::SsssPermitter>, port: u16) {
    let bind_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(
        listener,
        make_router(AppState {
            store,
            sssss: sssss.map(|ssss| (ssss.chain, ssss)).collect(),
        }),
    )
    .await
    .unwrap();
}

fn make_router<S: Store>(state: AppState<S>) -> Router {
    Router::new()
        .route("/", any(root))
        .nest(
            "v1",
            Router::new()
                .nest(
                    "/permits/:chain/:registry/:identity",
                    Router::new()
                        .route("/", post(acqrel_identity))
                        .route("/", delete(acqrel_identity)),
                )
                .route(
                    "/v1/shares/omni/:chain/:registry/:identity",
                    get(get_omni_key_share),
                ),
        )
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(
            cors::CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::DELETE])
                .allow_origin(cors::Any)
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                    HeaderName::from_static("signature"),
                ]),
        )
        .with_state(state)
}

async fn root() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn acqrel_identity<S: Store>(
    method: Method,
    Path((chain, registry, identity)): Path<(ChainId, Address, IdentityId)>,
    State(AppState { store, sssss }): State<AppState<S>>,
    Json(AcqRelIdentityRequest {
        duration,
        authorization,
        context,
        permitter,
        recipient,
    }): Json<AcqRelIdentityRequest>,
) -> Result<StatusCode, Error> {
    let ssss = sssss
        .get(&chain)
        .ok_or_else(|| Error::BadRequest(format!("unsupported chain: {chain}")))?;

    let policy_bytes = retry_times(
        || store.get_verifier(PermitterLocator::new(chain, permitter), identity),
        3,
    )
    .await
    .map_err(anyhow::Error::from)?
    .ok_or_else(|| Error::NotFound("policy not found".into()))?;

    let identity_locator = IdentityLocator {
        chain,
        registry,
        id: identity,
    };
    let verification = verify::verify(
        &policy_bytes,
        match method {
            Method::POST => verify::RequestKind::Grant { duration },
            Method::DELETE => verify::RequestKind::Revoke,
            _ => unreachable!(),
        },
        identity_locator,
        recipient,
        &context,
        &authorization,
    )
    .await
    .map_err(|e| Error::Unauthorized(e.to_string()))?;

    let share = ShareId {
        identity: identity_locator,
        version: 0,
    };

    // TODO: call permitter to approve or revoke

    if ssss.upstream().await.map_err(anyhow::Error::from)? != registry {
        return Ok(StatusCode::ACCEPTED);
    }
    // If the upstream of the SsssPermitter is the identity registry, it's
    // safe to optimistically create the permit rather than waiting for quorum.
    match method {
        Method::POST => {
            let expiry = verification
                .expiry
                .ok_or_else(|| Error::Unauthorized("verification failed".into()))?;
            retry_times(
                || store.create_permit(share, recipient, expiry, verification.nonce.clone()),
                3,
            )
            .await
            .map_err(anyhow::Error::from)?
            .ok_or_else(|| {
                Error::Unauthorized(
                    "permit not created. maybe the nonce was already consumed".into(),
                )
            })?;
            Ok(StatusCode::CREATED)
        }
        Method::DELETE => {
            retry_times(|| store.delete_permit(share, recipient), 3)
                .await
                .map_err(anyhow::Error::from)?;
            Ok(StatusCode::NO_CONTENT)
        }
        _ => unreachable!(),
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AcqRelIdentityRequest {
    #[serde(default)]
    duration: u64,
    #[serde(with = "hex::serde")]
    authorization: Vec<u8>,
    #[serde(default, with = "hex::serde")]
    context: Vec<u8>,
    permitter: Address,
    recipient: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AcqRelIdentityResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    permit: Option<Permit>,
}

fn extract_header<T>(
    headers: &HeaderMap,
    header: &'static str,
    mapper: impl FnOnce(&str) -> anyhow::Result<T>,
) -> Result<T, Error> {
    let header_str = headers
        .get(header)
        .ok_or(Error::BadRequest(format!("missing `{header}` header")))?
        .to_str()
        .map_err(|_| Error::BadRequest(format!("invalid `${header}` header")))?;
    mapper(header_str).map_err(|e| Error::BadRequest(format!("invalid `{header}` header: {e}")))
}

async fn get_omni_key_share<S: Store>(
    headers: HeaderMap,
    Path((chain, registry, identity)): Path<(ChainId, Address, IdentityId)>,
    Query(GetOmniKeyQuery { share_version }): Query<GetOmniKeyQuery>,
    State(AppState { store, .. }): State<AppState<S>>,
) -> Result<Json<OmniKeyResponse>, Error> {
    let sig: ethers::types::Signature = extract_header(&headers, "signature", |h| {
        let sig_bytes = hex::decode(h)?;
        Ok(sig_bytes.as_slice().try_into()?)
    })?;
    let host = extract_header(&headers, "host", |h: &str| Ok(h.to_string()))?;

    let req721_hash = H256(
        OmniKeyRequest721 {
            audience: host,
            chain,
            registry,
            identity: identity.0,
            share_version,
        }
        .encode_eip712()
        .unwrap(),
    );
    let requester = sig
        .recover(req721_hash)
        .map_err(|_| Error::Forbidden("invalid eip712 signature".into()))?;
    let share_id = ShareId {
        identity: IdentityLocator {
            chain,
            registry,
            id: identity,
        },
        version: share_version,
    };
    store
        .read_permit(share_id, requester)
        .await?
        .ok_or_else(|| Error::Unauthorized("no acceptable permit found".into()))?;
    let share = store
        .get_share(share_id)
        .await?
        .ok_or_else(|| Error::NotFound("share not found".into()))?;
    Ok(Json(OmniKeyResponse {
        share: share.to_vec().into(),
    }))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetOmniKeyQuery {
    #[serde(rename = "version")]
    share_version: u64,
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
    audience: String,
    chain: u64,
    registry: Address,
    identity: H256,
    share_version: u64,
}

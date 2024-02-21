use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddrV4},
};

use axum::{
    extract::{Path, Query, State},
    http::{header, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{any, delete, get, post},
    Json, Router,
};
use axum_extra::{headers, TypedHeader};
use ethers::{
    middleware::Middleware,
    types::{transaction::eip712::Eip712 as _, Address, Bytes, H256},
};
use futures::TryFutureExt as _;
use serde::{Deserialize, Serialize};
use tower_http::cors;

use crate::{eth::SsssPermitter, store::Store, types::*, utils::retry_times, verify};

#[derive(Clone)]
struct AppState<M: Middleware, S> {
    store: S,
    sssss: HashMap<ChainId, SsssPermitter<M>>,
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

pub async fn serve<M: Middleware + Clone + 'static, S: Store>(
    store: S,
    sssss: impl Iterator<Item = SsssPermitter<M>>,
    port: u16,
) {
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

fn make_router<M: Middleware + Clone + 'static, S: Store>(state: AppState<M, S>) -> Router {
    Router::new()
        .route("/", any(root))
        .nest(
            "/v1",
            Router::new()
                .nest(
                    "/permits/:chain/:registry/:identity",
                    Router::new()
                        .route("/", post(acqrel_identity))
                        .route("/", delete(acqrel_identity)),
                )
                .route(
                    "/shares/omni/:chain/:registry/:identity",
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
                    SIGNATURE_HEADER_NAME.clone(),
                ]),
        )
        .with_state(state)
}

async fn root() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn acqrel_identity<M: Middleware + 'static, S: Store>(
    method: Method,
    Path((chain, registry, identity)): Path<(ChainId, Address, IdentityId)>,
    State(AppState { store, sssss }): State<AppState<M, S>>,
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
    .ok_or_else(|| Error::NotFound("policy".into()))?;

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
        &authorization,
        &context,
        Default::default(), // TODO: request signing
    )
    .await
    .map_err(|e| Error::Unauthorized(e.to_string()))?;

    let share = ShareId {
        identity: identity_locator,
        version: 1,
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
                    "permit not created. maybe there is already a permit or this request's nonce \
                     was already consumed"
                        .into(),
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

#[derive(Clone, Debug, Deserialize)]
struct AcqRelIdentityRequest {
    #[serde(default)]
    duration: u64,
    authorization: Bytes,
    context: Bytes,
    permitter: Address,
    recipient: Address,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AcqRelIdentityResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    permit: Option<Permit>,
}

struct SignatureHeader(ethers::types::Signature);

static SIGNATURE_HEADER_NAME: header::HeaderName = header::HeaderName::from_static("signature");

impl headers::Header for SignatureHeader {
    fn name() -> &'static header::HeaderName {
        &SIGNATURE_HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i header::HeaderValue>,
    {
        let sig_hex = values.next().ok_or_else(headers::Error::invalid)?;
        let sig_bytes: Bytes = sig_hex
            .to_str()
            .ok()
            .and_then(|s| s.parse().ok())
            .ok_or_else(headers::Error::invalid)?;
        Ok(Self(
            (&*sig_bytes)
                .try_into()
                .map_err(|_| headers::Error::invalid())?,
        ))
    }

    fn encode<E: Extend<header::HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(
            header::HeaderValue::from_str(&format!("0x{}", hex::encode(self.0.to_vec()))).unwrap(),
        ));
    }
}

impl std::ops::Deref for SignatureHeader {
    type Target = ethers::types::Signature;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

async fn get_omni_key_share<M: Middleware, S: Store>(
    sig: TypedHeader<SignatureHeader>,
    host: TypedHeader<headers::Host>,
    Path((chain, registry, identity)): Path<(ChainId, Address, IdentityId)>,
    Query(GetOmniKeyQuery { share_version }): Query<GetOmniKeyQuery>,
    State(AppState { store, .. }): State<AppState<M, S>>,
) -> Result<Json<OmniKeyResponse>, Error> {
    let req721_hash = H256(
        OmniKeyRequest721 {
            audience: host.to_string(),
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

    retry_times(|| store.read_permit(share_id, requester), 3)
        .map_err(anyhow::Error::from)
        .await?
        .ok_or_else(|| Error::Unauthorized("no acceptable permit found".into()))?;

    let get_share = || retry_times(|| store.get_share(share_id), 3).map_err(anyhow::Error::from);
    let share = match get_share().await? {
        Some(share) => share,
        None => {
            retry_times(|| store.create_share(share_id.identity), 3)
                .await
                .map_err(anyhow::Error::from)?;
            get_share()
                .await?
                .ok_or_else(|| Error::NotFound("share".into()))?
        }
    };

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
    share: Bytes,
}

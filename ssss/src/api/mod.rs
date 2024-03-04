mod auth;

use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddrV4},
};

use axum::{
    extract::{Path, Query, Request, State},
    http::{header, Method, StatusCode, uri::Authority},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{any, delete, get, post, put},
    Json, Router,
};
use axum_extra::TypedHeader;
use ethers::{
    middleware::Middleware,
    types::{Address, Bytes},
};
use futures::TryFutureExt as _;
use serde::{Deserialize, Serialize};
use tower_http::cors;

use crate::{eth::SsssPermitter, store::Store, types::*, utils::retry_times, verify};

#[derive(Clone)]
struct AppState<M: Middleware, S> {
    store: S,
    sssss: HashMap<ChainId, SsssPermitter<M>>,
    host: Authority
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
    host: Authority,
) {
    let bind_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), host.port_u16().unwrap_or(443));
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(
        listener,
        make_router(AppState {
            store,
            sssss: sssss.map(|ssss| (ssss.chain, ssss)).collect(),
            host,
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
                    "/shares/:name/:chain/:registry/:identity",
                    get(get_share)
                        .layer(axum::middleware::from_fn_with_state(
                            state.clone(),
                            auth::permitted_requester,
                        ))
                        .layer(axum::middleware::from_fn(support_only_omni("share"))),
                )
                .nest(
                    "/keys/:name/:chain/:registry/:identity",
                    Router::new()
                        .route("/", put(put_key))
                        .route("/", get(get_key))
                        .route("/", delete(delete_key))
                        .layer(axum::middleware::from_fn_with_state(
                            state.clone(),
                            auth::permitted_requester,
                        ))
                        .layer(axum::middleware::from_fn(support_only_omni("key"))),
                )
                .layer(axum::middleware::from_fn_with_state(state.clone(), auth::escrin1)),
        )
        .with_state(state)
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(
            cors::CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::DELETE])
                .allow_origin(cors::Any)
                .allow_headers([
                    header::CONTENT_TYPE,
                    header::AUTHORIZATION,
                    auth::SIGNATURE_HEADER_NAME.clone(),
                    auth::REQUESTER_HEADER_NAME.clone(),
                ]),
        )
}

fn support_only_omni(
    item: &'static str,
) -> impl (Fn(
    Path<(String,)>,
    Request,
    Next,
) -> futures::future::BoxFuture<'static, Result<Response, Error>>)
       + Clone {
    move |Path((name,)), req, next| {
        Box::pin(async move {
            if name != "omni" {
                return Err(Error::NotFound(format!("{item} with name {name}")));
            }
            Ok(next.run(req).await)
        })
    }
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
                || {
                    store.create_permit(
                        identity_locator,
                        recipient,
                        expiry,
                        verification.nonce.clone(),
                    )
                },
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
            retry_times(|| store.delete_permit(identity_locator, recipient), 3)
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

async fn get_share<M: Middleware, S: Store>(
    Path((name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetShareQuery { version }): Query<GetShareQuery>,
    TypedHeader(auth::Requester(requester)): TypedHeader<auth::Requester>,
    State(AppState { store, .. }): State<AppState<M, S>>,
) -> Result<Json<ShareResponse>, Error> {
    let share = retry_times(
        || {
            store.get_share(ShareId {
                identity: IdentityLocator {
                    chain,
                    registry,
                    id: identity,
                },
                version,
            })
        },
        3,
    )
    .map_err(anyhow::Error::from)
    .await?
    .ok_or_else(|| Error::NotFound("share".into()))?;

    Ok(Json(ShareResponse {
        share: share.to_vec().into(),
    }))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetShareQuery {
    version: u64,
}

#[derive(Clone, Debug, Serialize)]
struct ShareResponse {
    share: Bytes,
}

async fn put_key<M: Middleware, S: Store>(
    Path((name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetKeyQuery { version }): Query<GetKeyQuery>,
    TypedHeader(auth::Requester(requester)): TypedHeader<auth::Requester>,
    State(AppState { store, .. }): State<AppState<M, S>>,
) -> Result<StatusCode, Error> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

async fn get_key<M: Middleware, S: Store>(
    Path((name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetShareQuery {
        version: share_version,
    }): Query<GetShareQuery>,
    TypedHeader(auth::Requester(requester)): TypedHeader<auth::Requester>,
    State(AppState { store, .. }): State<AppState<M, S>>,
) -> Result<Json<KeyResponse>, Error> {
    Ok(Json(KeyResponse {
        key: Default::default(),
    }))
}

async fn delete_key<M: Middleware, S: Store>(
    Path((name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetKeyQuery { version }): Query<GetKeyQuery>,
    TypedHeader(auth::Requester(requester)): TypedHeader<auth::Requester>,
    State(AppState { store, .. }): State<AppState<M, S>>,
) -> Result<StatusCode, Error> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct GetKeyQuery {
    version: u64,
}

#[derive(Clone, Debug, Serialize)]
struct KeyResponse {
    key: Bytes,
}

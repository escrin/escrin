mod auth;

use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddrV4},
};

use aes_gcm_siv::AeadInPlace as _;
use axum::{
    extract::{Path, Query, Request, State},
    http::{header, uri::Authority, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{any, delete, get, post, put},
    Json, Router,
};
use axum_extra::{headers::Header as _, TypedHeader};
use ethers::{middleware::Middleware, types::Address};
use futures_util::TryFutureExt as _;
use p384::elliptic_curve::JwkEcKey;
use ssss::identity::{self, Identity};
use tower_http::cors;

use crate::{
    eth::SsssHub,
    store::Store,
    types::{api::*, *},
    utils::retry_times,
    verify,
};

#[derive(Clone)]
struct AppState<M: Middleware, S> {
    store: S,
    sssss: HashMap<ChainId, SsssHub<M>>,
    host: Authority,
    persistent_identity_jwk: JwkEcKey,
    ephemeral_identity: Identity,
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

pub async fn serve<M: Middleware + Clone + 'static, S: Store>(
    store: S,
    sssss: impl Iterator<Item = SsssHub<M>>,
    host: Authority,
    identity_jwk: JwkEcKey,
) {
    assert!(identity_jwk.is_public_key());
    let bind_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), host.port_u16().unwrap_or(443));
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(
        listener,
        make_router(AppState {
            store,
            sssss: sssss.map(|ssss| (ssss.chain, ssss)).collect(),
            host,
            persistent_identity_jwk: identity_jwk,
            ephemeral_identity: Identity::ephemeral(),
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
                .route("/identity", get(get_ssss_identity))
                .nest(
                    "/permits/:chain/:registry/:identity",
                    Router::new()
                        .route("/", post(acqrel_identity))
                        .route("/", delete(acqrel_identity))
                        .layer(axum::middleware::from_fn_with_state(
                            state.host.clone(),
                            auth::escrin1,
                        )),
                )
                .route(
                    "/shares/:name/:chain/:registry/:identity",
                    get(get_share)
                        .layer(axum::middleware::from_fn_with_state(
                            state.store.clone(),
                            auth::permitted_requester::<S>,
                        ))
                        .layer(axum::middleware::from_fn_with_state(
                            state.host.clone(),
                            auth::escrin1,
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
                            state.store.clone(),
                            auth::permitted_requester::<S>,
                        ))
                        .layer(axum::middleware::from_fn_with_state(
                            state.host.clone(),
                            auth::escrin1,
                        ))
                        .layer(axum::middleware::from_fn(support_only_omni("key"))),
                ),
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
                    SignatureHeader::name().clone(),
                    RequesterHeader::name().clone(),
                    RequesterPublicKeyHeader::name().clone(),
                ]),
        )
}

fn support_only_omni(
    item: &'static str,
) -> impl (Fn(
    Path<(String,)>,
    Request,
    Next,
) -> futures_util::future::BoxFuture<'static, Result<Response, Error>>)
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

async fn get_ssss_identity<M: Middleware + 'static, S: Store>(
    State(AppState {
        persistent_identity_jwk,
        ephemeral_identity,
        ..
    }): State<AppState<M, S>>,
) -> Json<IdentityResponse> {
    Json(IdentityResponse {
        persistent: persistent_identity_jwk,
        ephemeral: ephemeral_identity.public_key().to_jwk(),
    })
}

async fn acqrel_identity<M: Middleware + 'static, S: Store>(
    method: Method,
    Path((chain, registry, identity)): Path<(ChainId, Address, IdentityId)>,
    State(AppState { store, sssss, .. }): State<AppState<M, S>>,
    relayer: Option<TypedHeader<RequesterHeader>>,
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
        relayer.map(|r| r.0 .0),
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

async fn get_share<M: Middleware, S: Store>(
    Path((_name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetShareQuery { version }): Query<GetShareQuery>,
    requester_pk: Option<TypedHeader<RequesterPublicKeyHeader>>,
    State(AppState {
        store,
        ephemeral_identity,
        ..
    }): State<AppState<M, S>>,
) -> Result<Json<ShareResponse>, Error> {
    let SecretShare { index, share } = retry_times(
        || {
            store.get_share(ShareId {
                secret_name: "omni".into(),
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

    let (format, share) = match requester_pk {
        Some(pk) => {
            let cipher =
                ephemeral_identity.derive_shared_cipher(*pk.0, identity::GET_SHARE_DOMAIN_SEP);
            let mut nonce = aes_gcm_siv::Nonce::default();
            rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce);
            let mut enc_share = (*share).clone();
            cipher
                .encrypt_in_place(&nonce, &[], &mut enc_share)
                .map_err(|e| Error::Unhandled(anyhow::anyhow!("encryption error: {e}")))?;
            (
                ShareResponseFormat::EncAes256GcmSiv {
                    nonce: nonce.into(),
                },
                enc_share,
            )
        }
        None => (ShareResponseFormat::Plain, (*share).clone()),
    };

    Ok(Json(ShareResponse {
        format,
        ss: WrappedSecretShare { index, share },
    }))
}

async fn put_key<M: Middleware, S: Store>(
    Path((name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetKeyQuery { version }): Query<GetKeyQuery>,
    State(AppState { store, .. }): State<AppState<M, S>>,
    Json(PutKeyRequest { key }): Json<PutKeyRequest>,
) -> Result<StatusCode, Error> {
    let created = store
        .put_key(
            KeyId {
                name,
                identity: IdentityLocator {
                    chain,
                    registry,
                    id: identity,
                },
                version,
            },
            key,
        )
        .await?;
    Ok(if created {
        StatusCode::CREATED
    } else {
        StatusCode::CONFLICT
    })
}

async fn get_key<M: Middleware, S: Store>(
    Path((name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetKeyQuery { version }): Query<GetKeyQuery>,
    State(AppState { store, .. }): State<AppState<M, S>>,
) -> Result<Json<KeyResponse>, Error> {
    let key = store
        .get_key(KeyId {
            name,
            identity: IdentityLocator {
                chain,
                registry,
                id: identity,
            },
            version,
        })
        .await?;
    match key {
        Some(key) => Ok(Json(KeyResponse {
            key: key.into_vec().into(),
        })),
        None => Err(Error::NotFound("key".into())),
    }
}

async fn delete_key<M: Middleware, S: Store>(
    Path((name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetKeyQuery { version }): Query<GetKeyQuery>,
    State(AppState { store, .. }): State<AppState<M, S>>,
) -> Result<StatusCode, Error> {
    store
        .delete_key_version(KeyId {
            name,
            identity: IdentityLocator {
                chain,
                registry,
                id: identity,
            },
            version,
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

mod middleware;

use std::net::{Ipv4Addr, SocketAddrV4};

use aes_gcm_siv::AeadInPlace as _;
use axum::{
    extract::{Path, Query, Request, State},
    http::{header, uri::Authority, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{any, delete, get, post, put},
    Json, Router,
};
use axum_extra::{either::Either, headers::Header as _, TypedHeader};
use ethers::{
    core::{
        k256::{self, elliptic_curve::sec1::FromEncodedPoint as _},
        types::transaction::eip712,
        utils::keccak256,
    },
    providers::Middleware,
    types::{transaction::eip712::Eip712 as _, Address},
};
use futures_util::TryFutureExt as _;
use ssss::keypair::{self, KeyPair, RotatingKeyPairProvider};
use tower_http::cors;
use vsss_rs::PedersenVerifierSet;

use crate::{
    backend::{Signer, Store},
    eth,
    types::{api::*, *},
    verify,
};

#[derive(Clone)]
struct AppState<B> {
    backend: B,
    host: Authority,
    providers: eth::Providers,
    kps: RotatingKeyPairProvider<B>,
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
    #[error("unsupported chain: {0}")]
    UnsupportedChain(ChainId),
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
            Self::UnsupportedChain(_) => StatusCode::MISDIRECTED_REQUEST,
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

pub async fn serve<B: Store + Signer>(backend: B, providers: eth::Providers, host: Authority) {
    let bind_addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), host.port_u16().unwrap_or(443));
    let listener = tokio::net::TcpListener::bind(bind_addr).await.unwrap();
    axum::serve(
        listener,
        make_router(AppState {
            backend: backend.clone(),
            host,
            providers,
            kps: RotatingKeyPairProvider::new(backend),
        }),
    )
    .await
    .unwrap();
}

fn make_router<S: Store + Signer>(state: AppState<S>) -> Router {
    Router::new()
        .route("/", any(root))
        .nest(
            "/v1",
            Router::new()
                .route("/identity", get(get_ssss_identity))
                .nest(
                    "/policies/:chain/:registry/:identity",
                    Router::new().route("/", post(set_policy)).layer(
                        axum::middleware::from_fn_with_state(
                            state.providers.clone(),
                            middleware::ensure_supported_chain,
                        ),
                    ),
                )
                .nest(
                    "/permits/:chain/:registry/:identity",
                    Router::new()
                        .route("/", post(acqrel_identity))
                        .route("/", delete(acqrel_identity))
                        .layer(axum::middleware::from_fn_with_state(
                            state.host.clone(),
                            middleware::escrin1,
                        ))
                        .layer(axum::middleware::from_fn_with_state(
                            state.providers.clone(),
                            middleware::ensure_supported_chain,
                        )),
                )
                .nest(
                    "/shares/:name/:chain/:registry/:identity",
                    Router::new()
                        .route("/", get(get_share))
                        .route("/", post(deal_share))
                        .route("/", delete(destroy_share))
                        .route("/commit", post(commit_share))
                        .layer(axum::middleware::from_fn_with_state(
                            state.providers.clone(),
                            middleware::permitted_requester,
                        ))
                        .layer(axum::middleware::from_fn_with_state(
                            state.host.clone(),
                            middleware::escrin1,
                        ))
                        .layer(axum::middleware::from_fn_with_state(
                            state.providers.clone(),
                            middleware::ensure_supported_chain,
                        ))
                        .layer(axum::middleware::from_fn(support_only_omni("share"))),
                )
                .nest(
                    "/secrets/:name/:chain/:registry/:identity",
                    Router::new()
                        .route("/", put(put_secret))
                        .route("/", get(get_secret))
                        .route("/", delete(delete_secret))
                        .layer(axum::middleware::from_fn_with_state(
                            state.providers.clone(),
                            middleware::permitted_requester,
                        ))
                        .layer(axum::middleware::from_fn_with_state(
                            state.host.clone(),
                            middleware::escrin1,
                        ))
                        .layer(axum::middleware::from_fn_with_state(
                            state.providers.clone(),
                            middleware::ensure_supported_chain,
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

async fn get_ssss_identity<S: Store + Signer>(
    State(AppState { backend, kps, .. }): State<AppState<S>>,
) -> Result<Json<IdentityResponse>, Error> {
    let latest_key_fut = kps
        .with_latest_key(|id, kp, expiry| {
            (
                id.to_string(),
                *kp.public_key(),
                (expiry.duration_since(std::time::UNIX_EPOCH))
                    .unwrap()
                    .as_secs(),
            )
        })
        .map_ok(|(key_id, pk, expiry)| EphemeralKey { key_id, pk, expiry });
    let signer_addr_fut = backend.signer_address();
    let (ephemeral, signer) = tokio::try_join!(latest_key_fut, signer_addr_fut)?;
    Ok(Json(IdentityResponse { ephemeral, signer }))
}

async fn set_policy<S: Store>(
    Path((chain, registry, identity)): Path<(ChainId, Address, IdentityId)>,
    State(AppState {
        providers, backend, ..
    }): State<AppState<S>>,
    Json(SetPolicyRequest { permitter, policy }): Json<SetPolicyRequest>,
) -> Result<StatusCode, Error> {
    let Some(provider) = providers.get(&chain) else {
        return Err(Error::UnsupportedChain(chain));
    };
    let expected_policy_hash = eth::SsssPermitter::new(permitter, provider.clone())
        .policy_hash(identity)
        .await
        .map_err(|e| Error::Unhandled(e.into()))?;

    let provided_policy_hash = keccak256(policy.get());
    if provided_policy_hash != expected_policy_hash.0 {
        return Err(Error::BadRequest(
            "provided policy did not match registered policy".into(),
        ));
    }

    backend
        .put_verifier(
            PermitterLocator { chain, permitter },
            IdentityLocator {
                chain,
                registry,
                id: identity,
            },
            policy.get().as_bytes().to_vec(),
        )
        .await
        .map_err(Error::Unhandled)?;

    Ok(StatusCode::NO_CONTENT)
}

async fn acqrel_identity<S: Store + Signer>(
    method: Method,
    Path((chain, registry, identity)): Path<(ChainId, Address, IdentityId)>,
    State(AppState {
        backend, providers, ..
    }): State<AppState<S>>,
    relayer: Option<TypedHeader<RequesterHeader>>,
    Json(AcqRelIdentityRequest {
        duration,
        authorization,
        context,
        permitter,
        recipient,
        base_block,
    }): Json<AcqRelIdentityRequest>,
) -> Result<Json<PermitResponse>, Error> {
    let Some(provider) = providers.get(&chain) else {
        return Err(Error::UnsupportedChain(chain));
    };
    let current_block = provider
        .get_block_number()
        .await
        .map_err(|e| Error::Unhandled(e.into()))?;
    if base_block > current_block.low_u64() {
        return Err(Error::BadRequest("base block is in the future".into()));
    }

    let policy_bytes = backend
        .get_verifier(
            PermitterLocator::new(chain, permitter),
            IdentityLocator {
                chain,
                registry,
                id: identity,
            },
        )
        .await
        .map_err(anyhow::Error::from)?
        .ok_or_else(|| Error::NotFound("policy".into()))?;

    let policy_hash = keccak256(&policy_bytes);
    let current_policy_hash = eth::SsssPermitter::new(permitter, provider.clone())
        .policy_hash(identity)
        .await
        .map_err(|e| Error::Unhandled(e.into()))?;

    if policy_hash != current_policy_hash.0 {
        return Err(Error::Unauthorized("policy not current".into()));
    }

    let identity_locator = IdentityLocator {
        chain,
        registry,
        id: identity,
    };
    let crate::verify::Verification {
        nonce,
        public_key,
        duration,
    } = verify::verify(
        &policy_bytes,
        match method {
            Method::POST => verify::RequestKind::Grant {
                duration: duration
                    .ok_or_else(|| Error::BadRequest("missing duration".to_string()))?,
            },
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

    let permit = SsssPermit {
        registry,
        identity: identity.0,
        recipient,
        grant: method == Method::POST,
        duration: duration.unwrap_or_default(),
        nonce: nonce.into(),
        pk: public_key.into(),
        baseblock: base_block.into(),
    };

    let permit_hash = eip712::EIP712WithDomain {
        domain: eip712::EIP712Domain {
            name: Some("SsssPermitter".into()),
            version: Some("1".into()),
            chain_id: Some(chain.into()),
            verifying_contract: Some(permitter),
            salt: None,
        },
        inner: permit.clone(),
    }
    .encode_eip712()
    .map_err(|e| Error::Unhandled(e.into()))?;

    let (signer, signature) = tokio::try_join!(
        backend.signer_address().map_err(Error::Unhandled),
        backend.sign(permit_hash.into()).map_err(Error::Unhandled)
    )?;
    Ok(Json(PermitResponse {
        permit,
        signer,
        signature,
    }))
}

async fn get_share<S: Store>(
    Path((name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetShareQuery { version }): Query<GetShareQuery>,
    requester_pk: Option<TypedHeader<RequesterPublicKeyHeader>>,
    State(AppState { backend, .. }): State<AppState<S>>,
) -> Result<Either<Json<EncryptedPayload>, Json<ShareBody>>, Error> {
    let ss = backend
        .get_share(ShareId {
            secret_name: name.clone(),
            identity: IdentityLocator {
                chain,
                registry,
                id: identity,
            },
            version,
        })
        .await?
        .ok_or_else(|| Error::NotFound("share".into()))?;

    let res = ShareBody { share: ss.into() };

    let Some(peer_pk) = requester_pk else {
        return Ok(Either::E2(Json(res)));
    };

    let mut payload = serde_json::to_vec(&res).map_err(|e| Error::Unhandled(e.into()))?;

    let ephemeral_identity = KeyPair::ephemeral();

    let mut nonce = aes_gcm_siv::Nonce::default();
    rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut nonce);
    ephemeral_identity
        .derive_shared_cipher(*peer_pk.0, keypair::GET_SHARE_DOMAIN_SEP)
        .encrypt_in_place(&nonce, &[], &mut payload)
        .map_err(|e| Error::Unhandled(anyhow::anyhow!("encryption error: {e}")))?;

    Ok(Either::E1(Json(EncryptedPayload {
        format: EncryptedPayloadFormat::P384EcdhAes256GcmSiv {
            curve: CurveP384,
            pk: *ephemeral_identity.public_key(),
            nonce: nonce.into(),
            recipient_key_id: Default::default(),
        },
        payload: payload.into(),
    })))
}

async fn deal_share<S: Store>(
    Path((name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetShareQuery { version }): Query<GetShareQuery>,
    State(AppState { backend, kps, .. }): State<AppState<S>>,
    Json(req): Json<MaybeEncryptedRequest<api::SecretShare>>,
) -> Result<StatusCode, Error> {
    let ss = match req {
        MaybeEncryptedRequest::Plain(ss) => ss,
        MaybeEncryptedRequest::Encrypted(EncryptedPayload { format, payload }) => {
            let EncryptedPayloadFormat::P384EcdhAes256GcmSiv {
                pk,
                nonce,
                recipient_key_id,
                ..
            } = format
            else {
                return Err(Error::BadRequest("unknown encrypted request format".into()));
            };
            let mut payload = Vec::from(payload.0);
            kps.with_key(&recipient_key_id, |kp| {
                kp.derive_shared_cipher(pk, keypair::DEAL_SHARES_DOMAIN_SEP)
                    .decrypt_in_place(&nonce.into(), &[], &mut payload)
            })
            .await
            .map_err(|_| Error::BadRequest("decryption failed".into()))?;
            serde_json::from_slice(&payload)
                .map_err(|e| Error::BadRequest(format!("invalid payload: {e}")))?
        }
    };

    let verifiers = ss
        .meta
        .commitments
        .iter()
        .map(|c| {
            let ep = k256::EncodedPoint::from_bytes(c).map_err(anyhow::Error::from)?;
            Option::from(k256::ProjectivePoint::from_encoded_point(&ep))
                .ok_or_else(|| anyhow::anyhow!("invalid curve point"))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| Error::BadRequest(format!("invalid commitment(s): {e}")))?;
    <Vec<_> as PedersenVerifierSet<_>>::pedersen_set_with_generators_and_verifiers(
        k256::ProjectivePoint::GENERATOR,
        *PEDERSEN_VSS_BLINDER_GENERATOR,
        &verifiers,
    )
    .verify_share_and_blinder::<u64, (u64, Vec<u8>)>(
        &(ss.meta.index, ss.share.0.to_vec()), // TODO: don't allocate
        &(ss.meta.index, ss.blinder.0.to_vec()),
    )
    .map_err(|_| Error::BadRequest("invalid share or blinder".into()))?;

    let share_put = backend
        .put_share(
            ShareId {
                identity: IdentityLocator {
                    chain,
                    registry,
                    id: identity,
                },
                secret_name: name,
                version,
            },
            crate::types::SecretShare {
                meta: ss.meta,
                share: Vec::from(ss.share.0).into(),
                blinder: Vec::from(ss.blinder.0).into(),
            },
        )
        .await?;

    share_put
        .then_some(StatusCode::CREATED)
        .ok_or_else(|| Error::BadRequest("incorrect version".into()))
}

async fn destroy_share<S: Store>(
    Path((name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetShareQuery { version }): Query<GetShareQuery>,
    State(AppState { backend, .. }): State<AppState<S>>,
) -> Result<StatusCode, Error> {
    backend
        .delete_share(ShareId {
            identity: IdentityLocator {
                chain,
                registry,
                id: identity,
            },
            secret_name: name,
            version,
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn commit_share<S: Store>(
    Path((name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetShareQuery { version }): Query<GetShareQuery>,
    State(AppState { backend, .. }): State<AppState<S>>,
) -> Result<StatusCode, Error> {
    backend
        .commit_share(ShareId {
            identity: IdentityLocator {
                chain,
                registry,
                id: identity,
            },
            secret_name: name,
            version,
        })
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn put_secret<S: Store>(
    Path((name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetKeyQuery { version }): Query<GetKeyQuery>,
    State(AppState { backend, .. }): State<AppState<S>>,
    Json(PutKeyRequest { key }): Json<PutKeyRequest>,
) -> Result<StatusCode, Error> {
    let created = backend
        .put_secret(
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

async fn get_secret<S: Store>(
    Path((name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetKeyQuery { version }): Query<GetKeyQuery>,
    State(AppState { backend, .. }): State<AppState<S>>,
) -> Result<Json<KeyResponse>, Error> {
    let key = backend
        .get_secret(KeyId {
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

async fn delete_secret<S: Store>(
    Path((name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    Query(GetKeyQuery { version }): Query<GetKeyQuery>,
    State(AppState { backend, .. }): State<AppState<S>>,
) -> Result<StatusCode, Error> {
    backend
        .delete_secret(KeyId {
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

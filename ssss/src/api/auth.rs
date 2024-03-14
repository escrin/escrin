use std::{
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    extract::{OriginalUri, Path, Request, State},
    http::{
        uri::{Authority, PathAndQuery, Uri},
        Method,
    },
    middleware::Next,
    response::Response,
};
use axum_extra::TypedHeader;
use ethers::types::{transaction::eip712::Eip712 as _, Address, Signature, H256};
use pin_project_lite::pin_project;
use tiny_keccak::{Hasher as _, Keccak};

use super::Error;
use crate::{
    store::Store,
    types::{api::*, *},
    utils::retry_times,
};

#[tracing::instrument(level = "info", skip_all)]
pub async fn permitted_requester<S: Store>(
    Path((_name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    TypedHeader(RequesterHeader(requester)): TypedHeader<RequesterHeader>,
    State(store): State<S>,
    req: Request,
    next: Next,
) -> Result<Response, Error> {
    let identity_locator = IdentityLocator {
        chain,
        registry,
        id: identity,
    };
    retry_times(|| store.read_permit(identity_locator, requester), 3)
        .await
        .map_err(anyhow::Error::from)?
        .ok_or_else(|| Error::Unauthorized("no acceptable permit found".into()))?;
    Ok(next.run(req).await)
}

#[tracing::instrument(level = "info", skip_all)]
pub async fn escrin1(
    method: Method,
    OriginalUri(uri): OriginalUri,
    sig: Option<TypedHeader<SignatureHeader>>,
    requester: Option<TypedHeader<RequesterHeader>>,
    State(host): State<Authority>,
    req: Request,
    next: Next,
) -> Result<Response, Error> {
    let Some(TypedHeader(RequesterHeader(requester))) = requester else {
        return Ok(next.run(req).await);
    };
    let Some(TypedHeader(SignatureHeader(sig))) = sig else {
        return Err(Error::Unauthorized(
            "Header of type `signature` was missing".into(),
        ));
    };
    Ok(next
        .run(match method {
            Method::OPTIONS => req,
            Method::PUT | Method::POST | Method::PATCH => req.map(|b| {
                axum::body::Body::new(SignatureChecker {
                    inner: b,
                    hasher: Some(Keccak::v256()),
                    host,
                    method,
                    uri,
                    sig,
                    requester,
                })
            }),
            Method::GET | Method::DELETE => {
                verify_sig(
                    method,
                    host,
                    uri.path_and_query()
                        .cloned()
                        .unwrap_or_else(|| PathAndQuery::from_static("")),
                    sig,
                    requester,
                    None,
                )?;
                req
            }
            m => return Err(Error::BadRequest(format!("unsupported method: {m}"))),
        })
        .await)
}

fn verify_sig(
    method: Method,
    host: Authority,
    path_and_query: PathAndQuery,
    sig: Signature,
    requester: Address,
    body: Option<H256>,
) -> Result<(), Error> {
    let req721_hash = H256(
        SsssRequest {
            method: method.to_string(),
            host: host.to_string(),
            path_and_query: path_and_query.to_string(),
            body: body.unwrap_or_default(),
        }
        .encode_eip712()
        .unwrap(),
    );
    let recovered_requester = sig
        .recover(req721_hash)
        .map_err(|_| Error::Forbidden("invalid eip712 signature".into()))?;
    if recovered_requester != requester {
        return Err(Error::Forbidden(
            "escrin1 signature validation failed".into(),
        ));
    }
    Ok(())
}

pin_project! {
    #[derive(Clone)]
    struct SignatureChecker<B> {
        #[pin]
        inner: B,
        hasher: Option<Keccak>,
        method: Method,
        host: Authority,
        uri: Uri,
        sig: Signature,
        requester: Address
    }
}

impl<B> http_body::Body for SignatureChecker<B>
where
    B: http_body::Body,
    B::Data: AsRef<[u8]>,
    B::Error: Into<anyhow::Error>,
{
    type Data = B::Data;
    type Error = Error;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<Self::Data>, Self::Error>>> {
        let this = self.project();
        let res = match this.inner.poll_frame(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(None) => {
                let mut body_hash = [0u8; 32];
                this.hasher
                    .take()
                    .ok_or_else(|| Error::BadRequest("body already consumed".into()))?
                    .finalize(&mut body_hash);

                match verify_sig(
                    this.method.clone(),
                    this.host.clone(),
                    this.uri
                        .path_and_query()
                        .cloned()
                        .unwrap_or_else(|| PathAndQuery::from_static("")),
                    *this.sig,
                    *this.requester,
                    Some(body_hash.into()),
                ) {
                    Ok(()) => None,
                    Err(e) => Some(Err(e)),
                }
            }
            Poll::Ready(Some(Ok(frame))) => {
                if let Some(data) = frame.data_ref() {
                    this.hasher
                        .as_mut()
                        .ok_or_else(|| Error::BadRequest("body already consumed".into()))?
                        .update(data.as_ref());
                }
                Some(Ok(frame))
            }
            Poll::Ready(Some(Err(err))) => Some(Err(Error::Unhandled(err.into()))),
        };

        Poll::Ready(res)
    }

    fn is_end_stream(&self) -> bool {
        self.inner.is_end_stream()
    }

    fn size_hint(&self) -> http_body::SizeHint {
        self.inner.size_hint()
    }
}

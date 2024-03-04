use std::{
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    extract::{OriginalUri, Path, Request, State},
    http::{header, Method, Uri},
    middleware::Next,
    response::Response,
};
use axum_extra::{headers, TypedHeader};
use ethers::{
    middleware::Middleware,
    types::{transaction::eip712::Eip712 as _, Address, Bytes, Signature, H256},
};
use pin_project_lite::pin_project;
use tiny_keccak::{Hasher as _, Keccak};

use super::{AppState, Error};
use crate::{store::Store, types::*, utils::retry_times};

pub async fn permitted_requester<M: Middleware + Clone + 'static, S: Store>(
    Path((_name, chain, registry, identity)): Path<(String, ChainId, Address, IdentityId)>,
    TypedHeader(Requester(requester)): TypedHeader<Requester>,
    State(AppState { store, .. }): State<AppState<M, S>>,
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

pub async fn escrin1<M: Middleware + Clone + 'static, S: Store>(
    method: Method,
    OriginalUri(uri): OriginalUri,
    TypedHeader(SignatureHeader(sig)): TypedHeader<SignatureHeader>,
    TypedHeader(Requester(requester)): TypedHeader<Requester>,
    State(AppState { host, .. }): State<AppState<M, S>>,
    req: Request,
    next: Next,
) -> Result<Response, Error> {
    if uri.authority() != Some(&host) {
        return Err(Error::Forbidden("incorrect audience".into()))
    }
    Ok(next
        .run(match method {
            Method::OPTIONS => req,
            Method::PUT | Method::POST | Method::PATCH => req.map(|b| {
                axum::body::Body::new(SignatureChecker {
                    inner: b,
                    hasher: Some(Keccak::v256()),
                    method,
                    uri,
                    sig,
                    requester,
                })
            }),
            Method::GET | Method::DELETE => {
                verify_sig(method, uri, sig, requester, None)?;
                req
            }
            m => return Err(Error::BadRequest(format!("unsupported method: {m}"))),
        })
        .await)
}

fn verify_sig(
    method: Method,
    uri: Uri,
    sig: Signature,
    requester: Address,
    body: Option<H256>,
) -> Result<(), Error> {
    let req721_hash = H256(
        SsssRequest {
            method: method.to_string(),
            uri: uri.to_string(),
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
                    *this.method,
                    this.uri.clone(),
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

pub struct SignatureHeader(Signature);

pub static SIGNATURE_HEADER_NAME: header::HeaderName = header::HeaderName::from_static("signature");

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
    type Target = Signature;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Requester(pub Address);

pub static REQUESTER_HEADER_NAME: header::HeaderName = header::HeaderName::from_static("requester");

impl headers::Header for Requester {
    fn name() -> &'static header::HeaderName {
        &REQUESTER_HEADER_NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i header::HeaderValue>,
    {
        Ok(Self(
            values
                .next()
                .ok_or_else(headers::Error::invalid)?
                .to_str()
                .ok()
                .and_then(|s| s.parse().ok())
                .ok_or_else(headers::Error::invalid)?,
        ))
    }

    fn encode<E: Extend<header::HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(
            header::HeaderValue::from_str(&format!("{:x}", self.0)).unwrap(),
        ));
    }
}

impl std::ops::Deref for Requester {
    type Target = Address;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

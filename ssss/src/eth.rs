use std::collections::HashMap;

use ethers::providers::{
    Http, Middleware as _, Provider, Quorum, QuorumProvider, WeightedProvider,
};
use futures::TryStreamExt;

pub type ChainId = u32;

pub async fn providers(
    rpcs: impl Iterator<Item = impl AsRef<str>>,
) -> Result<HashMap<ChainId, QuorumProvider<Provider<Http>>>, Error> {
    Ok(futures::stream::iter(rpcs.map(|rpc| {
        let rpc = rpc.as_ref();
        let url = url::Url::parse(rpc).map_err(|_| Error::UnsupportedRpc(rpc.into()))?;
        if url.scheme() != "http" {
            return Err(Error::UnsupportedRpc(rpc.into()));
        }
        Ok(Provider::new(Http::new(url)))
    }))
    .map_ok(|provider| async move { Ok((provider.get_chainid().await?.as_u32(), provider)) })
    .try_buffer_unordered(10)
    .try_fold(
        HashMap::<ChainId, Vec<Provider<Http>>>::new(),
        |mut providers, (chain_id, provider)| async move {
            providers.entry(chain_id).or_default().push(provider);
            Ok(providers)
        },
    )
    .await?
    .into_iter()
    .map(|(chain_id, providers)| {
        (
            chain_id,
            QuorumProvider::new(
                Quorum::Majority,
                providers.into_iter().map(WeightedProvider::new),
            ),
        )
    })
    .collect())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("contract call error: {0}")]
    Contract(#[from] ethers::contract::ContractError<Provider<Http>>),
    #[error("provider error: {0}")]
    Provider(#[from] ethers::providers::ProviderError),
    #[error("unsupported rpc url: {0}")]
    UnsupportedRpc(String),
}

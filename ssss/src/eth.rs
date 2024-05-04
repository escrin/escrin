use std::{collections::HashMap, sync::Arc};

use ethers::{
    providers::{self, JsonRpcClient as _},
    types::{Address, H256, U64},
};
use futures_util::TryStreamExt as _;

use crate::types::*;

ethers::contract::abigen!(IdentityRegistryContract, "../evm/abi/IdentityRegistry.json");
ethers::contract::abigen!(
    SsssPermitterContract,
    "../evm/abi/ExperimentalSsssPermitter.json"
);

#[derive(Clone)]
pub struct IdentityRegistry<M> {
    contract: IdentityRegistryContract<M>,
}

impl<M: providers::Middleware> IdentityRegistry<M> {
    pub fn new(address: Address, provider: Arc<M>) -> Self {
        Self {
            contract: IdentityRegistryContract::new(address, provider),
        }
    }

    pub async fn is_permitted(
        &self,
        address: Address,
        identity: IdentityId,
    ) -> Result<bool, Error<M>> {
        let permit = self
            .contract
            .read_permit(address, identity.0.into())
            .call()
            .await?;
        Ok(permit.expiry > crate::utils::now())
    }

    pub async fn registrant(&self, identity: IdentityId) -> Result<Address, Error<M>> {
        Ok(self
            .contract
            .get_registrant(identity.0.into())
            .call()
            .await?
            .0)
    }
}

#[derive(Clone)]
pub struct SsssPermitter<M> {
    contract: SsssPermitterContract<M>,
}

impl<M: providers::Middleware> SsssPermitter<M> {
    pub fn new(address: Address, provider: Arc<M>) -> Self {
        Self {
            contract: SsssPermitterContract::new(address, provider),
        }
    }

    pub async fn policy_hash(&self, identity: IdentityId) -> Result<H256, Error<M>> {
        Ok(self
            .contract
            .policy_hashes(identity.0.into())
            .call()
            .await?
            .into())
    }
}

pub type Providers = Arc<HashMap<ChainId, Provider>>;
pub type Provider =
    Arc<providers::Provider<providers::QuorumProvider<providers::RetryClient<providers::Http>>>>;

pub async fn providers(
    rpcs: impl Iterator<Item = impl AsRef<str>>,
) -> Result<Providers, Error<Provider>> {
    let providers = futures_util::stream::iter(rpcs.map(|rpc| {
        let rpc = rpc.as_ref();
        let url = url::Url::parse(rpc).map_err(|_| Error::UnsupportedRpc(rpc.into()))?;
        if url.scheme() != "http" && url.scheme() != "https" {
            return Err(Error::UnsupportedRpc(rpc.into()));
        }
        Ok(providers::RetryClient::new(
            providers::Http::new(url),
            Box::<providers::HttpRateLimitRetryPolicy>::default(),
            10,
            2_000,
        ))
    }))
    .map_ok(|provider| async move {
        let chain_id = provider
            .request::<[(); 0], U64>("eth_chainId", [])
            .await
            .map_err(ethers::providers::ProviderError::from)?
            .as_u64();
        Ok((chain_id, provider))
    })
    .try_buffer_unordered(10)
    .try_fold(
        HashMap::<ChainId, Vec<_>>::new(),
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
            Arc::new(providers::Provider::new(providers::QuorumProvider::new(
                providers::Quorum::Majority,
                providers.into_iter().map(providers::WeightedProvider::new),
            ))),
        )
    })
    .collect();
    Ok(Arc::new(providers))
}

#[derive(Debug, thiserror::Error)]
pub enum Error<M: providers::Middleware> {
    #[error("contract call error: {0}")]
    Contract(#[from] ethers::contract::ContractError<M>),
    #[error("provider error: {0}")]
    RpcProvider(M::Error),
    #[error("provider error: {0}")]
    Provider(#[from] ethers::providers::ProviderError),
    #[error("unsupported rpc url: {0}")]
    UnsupportedRpc(String),
}

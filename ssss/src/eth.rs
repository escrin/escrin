use std::{collections::HashMap, sync::Arc};

use ethers::{
    abi::{AbiEncode as _, Detokenize},
    contract::ContractCall,
    providers::{self, JsonRpcClient as _, Middleware},
    types::{Address, Bytes, Signature, TxHash, H256, U256, U64},
};
use futures_util::TryStreamExt as _;

use crate::types::*;

ethers::contract::abigen!(IdentityRegistryContract, "$ABI_DIR/IdentityRegistry.json");
ethers::contract::abigen!(SsssPermitterContract, "$ABI_DIR/SsssPermitter.json");

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
    provider: Arc<M>,
    chain: tokio::sync::OnceCell<ChainId>,
}

impl<M: providers::Middleware> SsssPermitter<M> {
    pub fn new(address: Address, provider: Arc<M>) -> Self {
        Self {
            contract: SsssPermitterContract::new(address, provider.clone()),
            provider,
            chain: Default::default(),
        }
    }

    pub async fn set_policy_hash(
        &self,
        identity: IdentityId,
        policy_hash: impl Into<[u8; 32]>,
    ) -> Result<TxHash, Error<M>> {
        self.send_tx(
            self.contract
                .set_policy_hash(identity.0.into(), policy_hash.into()),
        )
        .await
    }

    pub async fn set_approvers_root(
        &self,
        identity: IdentityId,
        signers_root: H256,
        threshold: u64,
    ) -> Result<TxHash, Error<M>> {
        self.send_tx(self.contract.set_approvers_root(
            identity.0.into(),
            signers_root.into(),
            threshold.into(),
        ))
        .await
    }

    pub async fn policy_hash(&self, identity: IdentityId) -> Result<H256, Error<M>> {
        Ok(self
            .contract
            .policy_hashes(identity.0.into())
            .call()
            .await?
            .into())
    }

    pub async fn acquire_identity(
        &self,
        permit: SsssPermit,
        threshold: u64,
        signer_proof: (Vec<H256>, Vec<bool>),
        signatures: Vec<(Address, Signature)>,
    ) -> Result<TxHash, Error<M>> {
        let signatures = signatures
            .into_iter()
            .map(|(addr, Signature { r, s, v })| {
                // Convert the signature to ERC-2098 format.
                let s = match v {
                    27 => s,
                    28 => s | U256::one() << 255,
                    _ => unreachable!(),
                };
                (addr, r, s)
            })
            .collect::<Vec<_>>();

        let context = (threshold, permit.nonce, permit.pk, permit.baseblock);
        let authorization = (signer_proof.0, signer_proof.1, signatures);

        self.send_tx(self.contract.acquire_identity(
            permit.identity.0,
            permit.recipient,
            permit.duration,
            context.encode().into(),
            authorization.encode().into(),
        ))
        .await
    }

    pub async fn acquire_identity_raw(
        &self,
        identity: IdentityId,
        requester: Address,
        duration: u64,
        context: impl Into<Bytes>,
        authorization: impl Into<Bytes>,
    ) -> Result<TxHash, Error<M>> {
        self.send_tx(self.contract.acquire_identity(
            identity.0.into(),
            requester,
            duration,
            context.into(),
            authorization.into(),
        ))
        .await
    }

    pub async fn release_identity_raw(
        &self,
        identity: IdentityId,
        requester: Address,
        context: impl Into<Bytes>,
        authorization: impl Into<Bytes>,
    ) -> Result<TxHash, Error<M>> {
        self.send_tx(self.contract.release_identity(
            identity.0.into(),
            requester,
            context.into(),
            authorization.into(),
        ))
        .await
    }

    async fn send_tx<T: Detokenize>(&self, call: ContractCall<M, T>) -> Result<TxHash, Error<M>> {
        let receipt = call
            .send()
            .await?
            .interval(match self.chain().await {
                Ok(1337 | 31337) => providers::DEFAULT_LOCAL_POLL_INTERVAL,
                _ => providers::DEFAULT_POLL_INTERVAL,
            })
            .await?
            .unwrap();
        match receipt.status.map(|s| s.as_u64()) {
            Some(1) => Ok(receipt.transaction_hash),
            _ => Err(ethers::contract::ContractError::Revert(Default::default()).into()),
        }
    }

    async fn chain(&self) -> Result<ChainId, Error<M>> {
        self.chain
            .get_or_try_init(|| async {
                self.provider
                    .get_chainid()
                    .await
                    .map(|c| c.low_u64())
                    .map_err(Error::RpcProvider)
            })
            .await
            .copied()
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

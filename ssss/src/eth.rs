use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use ethers::{
    abi::AbiDecode,
    contract::{ContractCall, EthLogDecode as _},
    providers::{self, JsonRpcClient as _},
    types::{Address, Bytes, Filter, Log, Transaction, TxHash, ValueOrArray, H256, U256, U64},
};
use futures_util::{future::BoxFuture, FutureExt, Stream, StreamExt as _, TryStreamExt as _};
use smallvec::{smallvec, SmallVec};
use tokio::sync::{Mutex, OnceCell};
use tracing::{trace, warn};

use crate::{
    types::*,
    utils::{retry, retry_if},
};

ethers::contract::abigen!(
    SsssHubContract,
    r"[
        event PolicyChange()
        event SharesDealt()

        function creationBlock() view returns (uint256)
        function upstream() view returns (address)
        function getIdentityRegistry() view returns (address)

        function setPolicy(bytes32 identity, bytes calldata config)

        function dealShares(bytes32 identity, string secretName, uint64 version, bytes pk, bytes32 nonce, bytes[] shares)
    ]"
);

#[derive(Clone)]
pub struct SsssHub<M> {
    pub chain: u64,
    pub address: Address,
    contract: SsssHubContract<M>,
    provider: Arc<M>,

    creation_block: Arc<OnceCell<u64>>,
    upstream: Arc<Mutex<(Address, Instant)>>,
}

impl<M: providers::Middleware> SsssHub<M> {
    pub fn new(chain: u64, address: Address, provider: M) -> Self {
        let provider = Arc::new(provider);
        Self {
            chain,
            address,
            contract: SsssHubContract::new(address, provider.clone()),
            provider,
            creation_block: Default::default(),
            upstream: Arc::new(Mutex::new((Address::zero(), Instant::now()))),
        }
    }

    pub async fn creation_block(&self) -> Result<u64, Error<M>> {
        match self.creation_block.get() {
            Some(b) => Ok(*b),
            None => {
                let b = self.contract.creation_block().call().await?.as_u64();
                self.creation_block.set(b).ok();
                Ok(b)
            }
        }
    }

    pub async fn upstream(&self) -> Result<Address, Error<M>> {
        let mut up = self.upstream.lock().await;
        if up.1 > Instant::now() {
            return Ok(up.0);
        }
        let r = self.contract.upstream().call().await?;
        *up = (r, Instant::now() + Duration::from_secs(5 * 60));
        Ok(r)
    }

    pub async fn registry(&self) -> Result<Address, Error<M>> {
        Ok(self.contract.get_identity_registry().call().await?)
    }

    pub async fn set_policy(
        &self,
        identity: IdentityId,
        config: Vec<u8>,
    ) -> Result<TxHash, Error<M>> {
        self.send_tx(self.contract.set_policy(identity.0.into(), config.into()))
            .await
    }

    pub async fn deal_shares_sss(
        &self,
        identity: IdentityId,
        version: u64,
        pk: impl Into<Bytes>,
        nonce: [u8; 32],
        shares: Vec<impl Into<Bytes>>,
    ) -> Result<TxHash, Error<M>> {
        self.send_tx(self.contract.deal_shares(
            identity.0.into(),
            "omni".into(),
            version,
            pk.into(),
            nonce,
            shares.into_iter().map(Into::into).collect(),
        ))
        .await
    }

    async fn send_tx(&self, call: ContractCall<M, ()>) -> Result<TxHash, Error<M>> {
        let receipt = call
            .send()
            .await?
            .interval(match self.chain {
                1337 | 31337 => providers::DEFAULT_LOCAL_POLL_INTERVAL,
                _ => providers::DEFAULT_POLL_INTERVAL,
            })
            .await?
            .unwrap();
        match receipt.status.map(|s| s.as_u64()) {
            Some(1) => Ok(receipt.transaction_hash),
            _ => Err(ethers::contract::ContractError::Revert(Default::default()).into()),
        }
    }

    pub fn events(
        &self,
        start_block: u64,
        stop_block: Option<u64>,
    ) -> impl Stream<Item = BoxFuture<SmallVec<[Event; 4]>>> {
        async_stream::stream!({
            for await block in self.blocks(start_block).await {
                yield self.get_block_events(block, self.address).boxed();
                yield futures_util::future::ready(smallvec![Event {
                    kind: EventKind::ProcessedBlock,
                    index: Default::default(),
                    tx: Default::default(),
                }])
                .boxed();
                if Some(block) == stop_block {
                    break;
                }
            }
        })
    }

    async fn blocks(&self, start_block: u64) -> impl Stream<Item = u64> + '_ {
        let init_block = retry(|| async {
            Ok::<_, Error<M>>(
                self.provider
                    .get_block_number()
                    .await
                    .map_err(Error::RpcProvider)?
                    .as_u64(),
            )
        })
        .await;
        async_stream::stream!({
            let mut current_block = start_block;
            loop {
                if current_block <= init_block {
                    yield current_block;
                } else {
                    self.wait_for_block(current_block).await;
                    yield current_block;
                }
                current_block += 1;
            }
        })
    }

    async fn wait_for_block(&self, block_number: u64) {
        trace!(block = block_number, "waiting for block");
        retry_if(
            || async {
                Ok::<_, Error<M>>(
                    self.provider
                        .get_block_number()
                        .await
                        .map_err(Error::RpcProvider)?
                        .as_u64(),
                )
            },
            |num| (num >= block_number).then_some(num),
        )
        .await;
        trace!(block = block_number, "waited for block");
    }

    async fn get_block_events(&self, block_number: u64, addr: Address) -> SmallVec<[Event; 4]> {
        retry(move || {
            let provider = self.provider.clone();
            let filter = Filter::new()
                .select(block_number)
                .address(ValueOrArray::Value(addr));
            async move { provider.get_logs(&filter).await }
        })
        .map(futures_util::stream::iter)
        .flatten_stream()
        .map(|log| async move { self.decode_permitter_event(log).await })
        .buffer_unordered(100)
        .filter_map(futures_util::future::ready)
        .collect::<SmallVec<[Event; 4]>>()
        .await
    }

    async fn decode_permitter_event(&self, log: Log) -> Option<Event> {
        let (block, tx, log_index) = match (
            log.block_number,
            log.transaction_hash,
            log.log_index,
            log.removed,
        ) {
            (Some(block), Some(tx), Some(index), None | Some(false)) => {
                (block.as_u64(), tx, index.as_u64())
            }
            _ => return None,
        };
        let raw_log = (log.topics, log.data.to_vec()).into();
        let event = match SsssHubContractEvents::decode_log(&raw_log) {
            Ok(event) => event,
            Err(e) => {
                warn!("failed to decode log: {e}");
                return None;
            }
        };
        let Transaction { input, .. } =
            retry_if(|| self.provider.get_transaction(tx), |tx| tx).await;
        let kind = match event {
            SsssHubContractEvents::PolicyChangeFilter(_) => {
                let (identity, config): (H256, Bytes) = AbiDecode::decode(&input[4..]).unwrap();
                EventKind::PolicyChange(PolicyChange {
                    identity: identity.into(),
                    config: config.to_vec(),
                })
            }
            SsssHubContractEvents::SharesDealtFilter(_) => {
                let (identity, secret_name, version, pk, nonce, shares): (
                    H256,
                    String,
                    U256,
                    Bytes,
                    H256,
                    Vec<Bytes>,
                ) = AbiDecode::decode(&input[4..]).unwrap();
                EventKind::SharesDealt(SharesDealt {
                    identity: identity.into(),
                    secret_name,
                    version: version.low_u64(),
                    scheme: SsScheme::Shamir {
                        pk: p384::PublicKey::from_sec1_bytes(&pk).ok()?,
                        nonce,
                        shares,
                    },
                })
            }
        };
        Some(Event {
            kind,
            tx: Some(tx),
            index: EventIndex { block, log_index },
        })
    }
}

type Providers = HashMap<ChainId, Provider>;
type Provider =
    providers::Provider<Arc<providers::QuorumProvider<providers::RetryClient<providers::Http>>>>;

pub async fn providers(
    rpcs: impl Iterator<Item = impl AsRef<str>>,
) -> Result<Providers, Error<Provider>> {
    Ok(futures_util::stream::iter(rpcs.map(|rpc| {
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
            providers::Provider::new(Arc::new(providers::QuorumProvider::new(
                providers::Quorum::Majority,
                providers.into_iter().map(providers::WeightedProvider::new),
            ))),
        )
    })
    .collect())
}

#[derive(Clone, Debug)]
pub struct Event {
    pub kind: EventKind,
    pub index: EventIndex,
    pub tx: Option<TxHash>,
}

#[derive(Clone, Debug)]
pub enum EventKind {
    PolicyChange(PolicyChange),
    SharesDealt(SharesDealt),
    ProcessedBlock,
}

#[derive(Clone, Debug)]
pub struct PolicyChange {
    pub identity: IdentityId,
    pub config: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct SharesDealt {
    pub identity: IdentityId,
    pub secret_name: String,
    pub version: u64,
    pub scheme: SsScheme,
}

#[derive(Clone, Debug)]
pub enum SsScheme {
    Shamir {
        pk: p384::PublicKey,
        nonce: H256,
        /// Encrypted secret shares. One of which belongs to this SSSS.
        shares: Vec<Bytes>,
    },
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

use std::{collections::HashMap, sync::Arc};

use ethers::{
    abi::AbiDecode,
    contract::EthLogDecode as _,
    providers::{
        Http, HttpRateLimitRetryPolicy, JsonRpcClient as _, Middleware as _,
        Provider as EthersProvider, Quorum, QuorumProvider, RetryClient, WeightedProvider,
    },
    types::{Address, Filter, Log, Transaction, TxHash, ValueOrArray, H256, U64},
};
use futures::{future::BoxFuture, FutureExt, Stream, StreamExt as _, TryStreamExt as _};
use smallvec::{smallvec, SmallVec};
use tracing::{trace, warn};

use crate::{
    types::{ChainId, EventIndex, IdentityId},
    utils::{retry, retry_if},
};

pub type Providers = HashMap<ChainId, Provider>;
pub type Provider = EthersProvider<QuorumProvider<RetryClient<Http>>>;

ethers::contract::abigen!(
    SsssPermitterContract,
    r"[
        event Configuration()
        event Unimplemented()

        function creationBlock() view returns (uint256)
    ]"
);

#[derive(Clone)]
pub struct SsssPermitter {
    pub chain: u64,
    pub address: Address,
    contract: SsssPermitterContract<Provider>,
    provider: Arc<Provider>,
}

impl SsssPermitter {
    pub fn new(chain: u64, address: Address, provider: Provider) -> Self {
        let provider = Arc::new(provider);
        Self {
            chain,
            address,
            contract: SsssPermitterContract::new(address, provider.clone()),
            provider,
        }
    }

    pub async fn creation_block(&self) -> Result<u64, Error> {
        Ok(self.contract.creation_block().call().await?.as_u64())
    }

    pub fn events(
        &self,
        start_block: u64,
        stop_block: Option<u64>,
    ) -> impl Stream<Item = BoxFuture<SmallVec<[Event; 4]>>> {
        async_stream::stream!({
            for await block in self.blocks(start_block).await {
                yield self.get_block_events(block, self.address).boxed();
                yield futures::future::ready(smallvec![Event {
                    kind: EventKind::ProcessedBlock,
                    index: Default::default(),
                    tx: Default::default(),
                }]).boxed();
                if Some(block) == stop_block {
                    break;
                }
            }
        })
    }

    async fn blocks(&self, start_block: u64) -> impl Stream<Item = u64> + '_ {
        let init_block =
            retry(|| async { Ok::<_, Error>(self.provider.get_block_number().await?.as_u64()) })
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
            || async { Ok::<_, Error>(self.provider.get_block_number().await?.as_u64()) },
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
        .map(futures::stream::iter)
        .flatten_stream()
        .map(|log| async move { self.decode_permitter_event(log).await })
        .buffer_unordered(100)
        .filter_map(futures::future::ready)
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
            (Some(block), Some(tx), Some(index), None) => (block.as_u64(), tx, index.as_u64()),
            _ => return None,
        };
        let raw_log = (log.topics, log.data.to_vec()).into();
        let event = match SsssPermitterContractEvents::decode_log(&raw_log) {
            Ok(event) => event,
            Err(e) => {
                warn!("failed to decode log: {e}");
                return None;
            }
        };
        let Transaction { input, .. } =
            retry_if(|| self.provider.get_transaction(tx), |tx| tx).await;
        let kind = match event {
            SsssPermitterContractEvents::ConfigurationFilter(_) => {
                let (identity, config): (H256, Vec<u8>) = AbiDecode::decode(input).unwrap();
                EventKind::Configuration(ConfigurationEvent {
                    identity: identity.into(),
                    config,
                })
            }
            _ => return None,
        };
        Some(Event {
            kind,
            tx: Some(tx),
            index: EventIndex { block, log_index },
        })
    }
}

pub async fn providers(rpcs: impl Iterator<Item = impl AsRef<str>>) -> Result<Providers, Error> {
    Ok(futures::stream::iter(rpcs.map(|rpc| {
        let rpc = rpc.as_ref();
        let url = url::Url::parse(rpc).map_err(|_| Error::UnsupportedRpc(rpc.into()))?;
        if url.scheme() != "http" {
            return Err(Error::UnsupportedRpc(rpc.into()));
        }
        Ok(RetryClient::new(
            Http::new(url),
            Box::<HttpRateLimitRetryPolicy>::default(),
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
            EthersProvider::new(QuorumProvider::new(
                Quorum::Majority,
                providers.into_iter().map(WeightedProvider::new),
            )),
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
    Configuration(ConfigurationEvent),
    ProcessedBlock,
}

#[derive(Clone, Debug)]
pub struct ConfigurationEvent {
    pub identity: IdentityId,
    pub config: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("contract call error: {0}")]
    Contract(#[from] ethers::contract::ContractError<Provider>),
    #[error("provider error: {0}")]
    Provider(#[from] ethers::providers::ProviderError),
    #[error("unsupported rpc url: {0}")]
    UnsupportedRpc(String),
}

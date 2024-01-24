use std::{collections::HashMap, sync::Arc};

use ethers::{
    abi::AbiDecode,
    contract::EthLogDecode as _,
    providers::{
        Http, HttpRateLimitRetryPolicy, JsonRpcClient as _, Middleware as _,
        Provider as EthersProvider, Quorum, QuorumProvider, RetryClient, WeightedProvider,
    },
    types::{Address, Filter, Log, Transaction, TxHash, ValueOrArray, U256, U64},
};
use futures::{future::BoxFuture, FutureExt, Stream, StreamExt as _, TryStreamExt as _};
use smallvec::{smallvec, SmallVec};
use tracing::{trace, warn};

use crate::utils::{retry, retry_if};

pub type ChainId = u64;

pub type Providers = HashMap<ChainId, Provider>;
pub type Provider = EthersProvider<QuorumProvider<RetryClient<Http>>>;

ethers::contract::abigen!(
    SsssPermitterContract,
    r"[
        event GrantPermitRequested()
        event RevokePermitRequested()
        function creationBlock() view returns (uint256)
    ]"
);

#[derive(Clone)]
pub struct SsssPermitter {
    addr: Address,
    contract: SsssPermitterContract<Provider>,
    provider: Arc<Provider>,
}

impl SsssPermitter {
    pub fn new(addr: Address, provider: Provider) -> Self {
        let provider = Arc::new(provider);
        Self {
            addr,
            contract: SsssPermitterContract::new(addr, provider.clone()),
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
                yield self.get_block_events(block, self.addr).boxed();
                yield futures::future::ready(smallvec![Event::ProcessedBlock(block)]).boxed();
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
        let (log_index, tx_hash) = match (log.log_index, log.transaction_hash, log.removed) {
            (Some(ix), Some(tx), None) => (ix.as_u64(), tx),
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
        let Transaction { from, input, .. } =
            retry_if(|| self.provider.get_transaction(tx_hash), |tx| tx).await;
        let (action, identity, recipient, context, authorization) = match event {
            SsssPermitterContractEvents::GrantPermitRequestedFilter(_) => {
                let (identity, recipient, duration, context, authorization): (
                    U256,
                    Address,
                    u64,
                    ethers::abi::Bytes,
                    ethers::abi::Bytes,
                ) = AbiDecode::decode(input).unwrap();
                (
                    PermitRequestKind::Grant { duration },
                    identity,
                    recipient,
                    context,
                    authorization,
                )
            }
            SsssPermitterContractEvents::RevokePermitRequestedFilter(_) => {
                let (identity, recipient, context, authorization): (
                    U256,
                    Address,
                    ethers::abi::Bytes,
                    ethers::abi::Bytes,
                ) = AbiDecode::decode(input).unwrap();
                (
                    PermitRequestKind::Revoke,
                    identity,
                    recipient,
                    context,
                    authorization,
                )
            }
        };

        Some(Event::PermitRequest(PermitRequestEvent {
            kind: action,
            identity,
            requester: from,
            recipient,
            context,
            authorization,
            tx: tx_hash,
            log_index,
        }))
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
pub enum Event {
    PermitRequest(PermitRequestEvent),
    Configuration(ConfigurationEvent),
    ProcessedBlock(u64),
}

#[derive(Clone, Debug)]
pub struct PermitRequestEvent {
    pub kind: PermitRequestKind,
    pub identity: U256,
    pub requester: Address,
    pub recipient: Address,
    pub context: Vec<u8>,
    pub authorization: Vec<u8>,
    pub tx: TxHash,
    pub log_index: u64,
}

#[derive(Clone, Debug)]
pub struct ConfigurationEvent {
    pub identity: U256,
    pub config: Vec<u8>,
    pub tx: TxHash,
    pub log_index: u64,
}

impl PermitRequestEvent {
    pub fn selector(&self) -> Option<String> {
        let sel: String = AbiDecode::decode(&self.context).ok()?;
        Some(sel)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PermitRequestKind {
    Grant { duration: u64 },
    Revoke,
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

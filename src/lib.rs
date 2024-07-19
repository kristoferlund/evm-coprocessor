#[allow(
    clippy::too_many_arguments,
    clippy::large_enum_variant,
    clippy::enum_variant_names
)]
pub mod evm_rpc;

use ic_cdk::api::call::{call_with_payment128, CallResult};
use std::{cell::RefCell, rc::Rc, time::Duration};
use thiserror::Error;

pub use evm_rpc::evm_rpc_types::*;

type Seconds = u64;
type Cycles = u128;

const DEFAULT_CYCLES: Cycles = 30_000_000_000;

#[derive(Clone)]
pub struct EthGetLogs {
    pub address: String,
    pub from_block: BlockTag,
    pub to_block: Option<BlockTag>,
    pub topics: Option<Vec<Topic>>,
    pub interval: Seconds,
    pub rpc_services: RpcServices,
    pub cycles: Cycles,
}

impl EthGetLogs {
    pub fn builder() -> EthGetLogsBuilder {
        EthGetLogsBuilder::default()
    }
}

#[derive(Error, Debug)]
pub enum EthGetLogsBuilderError {
    #[error("Required field missing: {0}")]
    RequiredFieldMissing(&'static str),
}

#[derive(Default)]
pub struct EthGetLogsBuilder {
    pub address: Option<String>,
    pub from_block: Option<BlockTag>,
    pub to_block: Option<BlockTag>,
    pub topics: Option<Vec<Topic>>,
    pub interval: Option<Seconds>,
    pub rpc_services: Option<RpcServices>,
    pub cycles: Option<Cycles>,
}

impl EthGetLogsBuilder {
    pub fn address(mut self, address: &str) -> Self {
        self.address = Some(address.to_string());
        self
    }

    pub fn from_block(mut self, block: BlockTag) -> Self {
        self.from_block = Some(block);
        self
    }

    pub fn to_block(mut self, block: BlockTag) -> Self {
        self.to_block = Some(block);
        self
    }

    pub fn topics(mut self, topics: Vec<Topic>) -> Self {
        self.topics = Some(topics);
        self
    }

    pub fn interval(mut self, interval: u64) -> Self {
        self.interval = Some(interval);
        self
    }

    pub fn rpc_services(mut self, rpc_services: RpcServices) -> Self {
        self.rpc_services = Some(rpc_services);
        self
    }

    pub fn cycles(mut self, cycles: Cycles) -> Self {
        self.cycles = Some(cycles);
        self
    }

    pub fn run(self, func: impl FnMut(LogEntry) + 'static) -> Result<(), EthGetLogsBuilderError> {
        if self.from_block.is_none() {
            return Err(EthGetLogsBuilderError::RequiredFieldMissing("from_block"));
        }

        let interval = self
            .interval
            .ok_or(EthGetLogsBuilderError::RequiredFieldMissing("interval"))?;

        if self.rpc_services.is_none() {
            return Err(EthGetLogsBuilderError::RequiredFieldMissing("rpc_services"));
        }

        let subscription = EthGetLogs {
            address: self.address.unwrap(),
            from_block: self.from_block.unwrap(),
            to_block: self.to_block,
            topics: self.topics,
            interval: self.interval.unwrap(),
            rpc_services: self.rpc_services.unwrap(),
            cycles: self.cycles.unwrap_or(DEFAULT_CYCLES),
        };

        let func = Rc::new(RefCell::new(func));

        ic_cdk_timers::set_timer_interval(Duration::from_secs(interval), move || {
            eth_get_logs(subscription.clone(), Rc::clone(&func));
        });

        Ok(())
    }
}

pub fn eth_get_logs(subscription: EthGetLogs, func: Rc<RefCell<impl FnMut(LogEntry) + 'static>>) {
    ic_cdk::spawn(async move {
        let call_result: CallResult<(MultiGetLogsResult,)> = call_with_payment128(
            evm_rpc_types.0,
            "eth_getLogs",
            (
                subscription.rpc_services,
                1,
                GetLogsArgs {
                    addresses: vec![subscription.address.clone()],
                    fromBlock: Some(subscription.from_block),
                    toBlock: subscription.to_block,
                    topics: subscription.topics,
                },
            ),
            subscription.cycles,
        )
        .await;

        match call_result {
            Ok((MultiGetLogsResult::Consistent(get_logs_result),)) => match get_logs_result {
                GetLogsResult::Ok(logs) => {
                    for log in logs {
                        (func.borrow_mut())(log);
                    }
                }
                GetLogsResult::Err(err) => {
                    ic_cdk::println!("Get logs result is an error: {:?}", err);
                }
            },
            Ok((inconsistent,)) => {
                ic_cdk::println!("Get logs result is inconsistent: {:?}", inconsistent);
            }
            Err(err) => {
                ic_cdk::println!("Error fetching logs: {:?}", err);
            }
        }
    });
}

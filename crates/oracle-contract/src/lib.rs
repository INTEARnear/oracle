#![cfg_attr(not(feature = "contract"), allow(unused_imports, dead_code))]

pub mod balance;
pub mod consumer;
pub mod fees;
pub mod producer;

use consumer::{Consumer, ConsumerId, RequestId};
use near_sdk::{near, store::LookupMap, BorshStorageKey};
use producer::{Producer, ProducerId};

#[derive(BorshStorageKey)]
#[near(serializers=[borsh])]
enum StorageKey {
    Producers,
    Consumers,
    PendingRequests {
        producer: ConsumerId,
    },
    ConsumerNearBalanceProducer {
        consumer: ConsumerId,
    },
    ConsumerFtBalances {
        consumer: ConsumerId,
    },
    ConsumerFtBalancesProducers {
        consumer: ConsumerId,
    },
    ConsumerFtBalancesProducer {
        consumer: ConsumerId,
        producer: ProducerId,
    },
}

// TODO: Storage management
#[cfg(feature = "contract")]
#[near(contract_state)]
pub struct Oracle {
    producers: LookupMap<ProducerId, Producer>,
    consumers: LookupMap<ConsumerId, Consumer>,
    next_request_id: RequestId,
}

#[cfg(feature = "contract")]
impl Default for Oracle {
    fn default() -> Self {
        Self {
            producers: LookupMap::new(StorageKey::Producers),
            consumers: LookupMap::new(StorageKey::Consumers),
            next_request_id: 0.into(),
        }
    }
}

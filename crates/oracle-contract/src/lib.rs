mod balance;
mod consumer;
mod producer;

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
#[near(contract_state)]
pub struct Contract {
    producers: LookupMap<ProducerId, Producer>,
    consumers: LookupMap<ConsumerId, Consumer>,
    next_request_id: RequestId,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            producers: LookupMap::new(StorageKey::Producers),
            consumers: LookupMap::new(StorageKey::Consumers),
            next_request_id: 0.into(),
        }
    }
}

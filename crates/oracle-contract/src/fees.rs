use near_sdk::{env, json_types::U128, near, NearToken, Promise};

use crate::{balance::FtId, consumer::ConsumerId, producer::ProducerId, Oracle, OracleExt};

/// Fees are set by producers. If the consumer's balance is less
/// than prepaid_amount, the request will be rejected without a log generated.
/// The producer may choose to refund a part of the fee if the request
/// was successful. If the indexer times out, the fee is refunded fully.
#[derive(Clone, PartialEq, Debug)]
#[near(serializers=[borsh, json])]
pub enum ProducerFee {
    None,
    Near { prepaid_amount: NearToken },
    FungibleToken { token: FtId, prepaid_amount: U128 },
}

/// A fee that the consumer has paid for a request. Can be partially
/// refunded by the producer in response.
#[near(serializers=[json, borsh])]
#[derive(Clone)]
pub enum PrepaidFee {
    None,
    Near {
        amount: NearToken,
        is_for_specific_producer: bool,
    },
    FungibleToken {
        token: FtId,
        amount: U128,
        is_for_specific_producer: bool,
    },
}

#[near]
impl Oracle {
    pub fn get_fee(&self, producer_id: &ProducerId) -> Option<ProducerFee> {
        self.producers
            .get(producer_id)
            .map(|producer| producer.fee.clone())
    }

    pub fn set_fee(&mut self, fee: ProducerFee) {
        self.producers
            .get_mut(&env::predecessor_account_id())
            .expect("Producer is not registered")
            .fee = fee;
    }
}

impl Oracle {
    pub fn try_charge_fee(
        &mut self,
        consumer_id: &ConsumerId,
        producer_id: &ProducerId,
        fee: &ProducerFee,
    ) -> Option<PrepaidFee> {
        match fee {
            ProducerFee::None => Some(PrepaidFee::None),
            ProducerFee::Near { prepaid_amount } => {
                let consumer = self
                    .consumers
                    .get_mut(consumer_id)
                    .expect("Consumer is not registered");

                if let Some(near_balance) = consumer.near_balance_producer.get_mut(producer_id) {
                    if *near_balance >= *prepaid_amount {
                        *near_balance = near_balance.checked_sub(*prepaid_amount).unwrap();
                        if near_balance.is_zero() {
                            consumer.near_balance_producer.remove(producer_id);
                        }
                        return Some(PrepaidFee::Near {
                            amount: *prepaid_amount,
                            is_for_specific_producer: true,
                        });
                    }
                }

                if consumer.near_balance >= *prepaid_amount {
                    consumer.near_balance =
                        consumer.near_balance.checked_sub(*prepaid_amount).unwrap();
                    return Some(PrepaidFee::Near {
                        amount: *prepaid_amount,
                        is_for_specific_producer: false,
                    });
                }

                None
            }
            ProducerFee::FungibleToken {
                token: _,
                prepaid_amount: _,
            } => unimplemented!(),
        }
    }

    pub fn refund_partially(
        &mut self,
        consumer_id: &ConsumerId,
        producer_id: &ProducerId,
        fee: &PrepaidFee,
        refund_amount: U128,
    ) {
        match fee {
            PrepaidFee::None => {}
            PrepaidFee::Near {
                amount: _,
                is_for_specific_producer,
            } => {
                let consumer = self
                    .consumers
                    .get_mut(consumer_id)
                    .expect("Consumer is not registered");

                if *is_for_specific_producer {
                    if let Some(near_balance) = consumer.near_balance_producer.get_mut(producer_id)
                    {
                        *near_balance = near_balance
                            .checked_add(NearToken::from_yoctonear(refund_amount.0))
                            .unwrap();
                    } else {
                        consumer.near_balance_producer.insert(
                            producer_id.clone(),
                            NearToken::from_yoctonear(refund_amount.0),
                        );
                    }
                } else {
                    consumer.near_balance = consumer
                        .near_balance
                        .checked_add(NearToken::from_yoctonear(refund_amount.0))
                        .unwrap();
                }
            }
            PrepaidFee::FungibleToken {
                token: _,
                amount: _,
                is_for_specific_producer: _,
            } => unimplemented!(),
        }
    }

    pub fn refund_fully(
        &mut self,
        consumer_id: &ConsumerId,
        producer_id: &ProducerId,
        fee: &PrepaidFee,
    ) {
        match fee {
            PrepaidFee::None => {}
            PrepaidFee::Near {
                amount,
                is_for_specific_producer,
            } => {
                let consumer = self
                    .consumers
                    .get_mut(consumer_id)
                    .expect("Consumer is not registered");

                if *is_for_specific_producer {
                    if let Some(near_balance) = consumer.near_balance_producer.get_mut(producer_id)
                    {
                        *near_balance = near_balance.checked_add(*amount).unwrap();
                    } else {
                        consumer
                            .near_balance_producer
                            .insert(producer_id.clone(), *amount);
                    }
                } else {
                    consumer.near_balance = consumer.near_balance.checked_add(*amount).unwrap();
                }
            }
            PrepaidFee::FungibleToken {
                token: _,
                amount: _,
                is_for_specific_producer: _,
            } => unimplemented!(),
        }
    }

    pub fn deposit_to_producer(
        &mut self,
        producer_id: ProducerId,
        fee: &PrepaidFee,
        refund_amount: Option<U128>,
    ) {
        match fee {
            PrepaidFee::None => {}
            PrepaidFee::Near {
                amount,
                is_for_specific_producer: _,
            } => {
                if let Some(deposit_amount) = amount.checked_sub(NearToken::from_yoctonear(
                    refund_amount.unwrap_or(U128(0)).0,
                )) {
                    Promise::new(producer_id.clone()).transfer(deposit_amount);
                } else {
                    env::panic_str("Refund amount is greater than prepaid amount")
                }
            }
            PrepaidFee::FungibleToken {
                token: _,
                amount: _,
                is_for_specific_producer: _,
            } => unimplemented!(),
        }
    }
}

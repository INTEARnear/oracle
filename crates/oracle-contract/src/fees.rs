use near_sdk::{env, json_types::U128, near, NearToken, Promise};

use crate::{balance::FtId, consumer::ConsumerId, producer::ProducerId};
#[cfg(feature = "contract")]
use crate::{Oracle, OracleExt};

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
#[derive(Debug, Clone)]
pub enum PrepaidFee {
    None,
    Near {
        amount: NearToken,
        payment_type: PaymentType,
    },
    FungibleToken {
        token: FtId,
        amount: U128,
        payment_type: PaymentType,
    },
}

#[near(serializers=[json, borsh])]
#[derive(Debug, Clone, PartialEq)]
pub enum PaymentType {
    ForSpecificProducer,
    ForAllProducers,
    AttachedToCall,
}

#[cfg(feature = "contract")]
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

#[cfg(feature = "contract")]
impl Oracle {
    pub fn try_charge_fee(
        &mut self,
        consumer_id: &ConsumerId,
        producer_id: &ProducerId,
        fee: &ProducerFee,
    ) -> Option<PrepaidFee> {
        match fee {
            ProducerFee::None => {
                if !env::attached_deposit().is_zero() {
                    panic!("No fee is required, but NEAR attached");
                }
                Some(PrepaidFee::None)
            }
            ProducerFee::Near { prepaid_amount } => {
                if !env::attached_deposit().is_zero() {
                    return Some(PrepaidFee::Near {
                        amount: env::attached_deposit(),
                        payment_type: PaymentType::AttachedToCall,
                    });
                }

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
                            payment_type: PaymentType::ForSpecificProducer,
                        });
                    }
                }

                if consumer.near_balance >= *prepaid_amount {
                    consumer.near_balance =
                        consumer.near_balance.checked_sub(*prepaid_amount).unwrap();
                    return Some(PrepaidFee::Near {
                        amount: *prepaid_amount,
                        payment_type: PaymentType::ForAllProducers,
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
                payment_type,
            } => match payment_type {
                PaymentType::ForSpecificProducer => {
                    let consumer = self
                        .consumers
                        .get_mut(consumer_id)
                        .expect("Consumer is not registered");

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
                }
                PaymentType::ForAllProducers => {
                    let consumer = self
                        .consumers
                        .get_mut(consumer_id)
                        .expect("Consumer is not registered");

                    consumer.near_balance = consumer
                        .near_balance
                        .checked_add(NearToken::from_yoctonear(refund_amount.0))
                        .unwrap();
                }
                PaymentType::AttachedToCall => {
                    Promise::new(consumer_id.clone())
                        .transfer(NearToken::from_yoctonear(refund_amount.0));
                }
            },
            PrepaidFee::FungibleToken {
                token: _,
                amount: _,
                payment_type: _,
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
                payment_type,
            } => {
                let consumer = self
                    .consumers
                    .get_mut(consumer_id)
                    .expect("Consumer is not registered");

                match payment_type {
                    PaymentType::ForSpecificProducer => {
                        if let Some(near_balance) =
                            consumer.near_balance_producer.get_mut(producer_id)
                        {
                            *near_balance = near_balance.checked_add(*amount).unwrap();
                        } else {
                            consumer
                                .near_balance_producer
                                .insert(producer_id.clone(), *amount);
                        }
                    }
                    PaymentType::ForAllProducers => {
                        consumer.near_balance = consumer.near_balance.checked_add(*amount).unwrap();
                    }
                    PaymentType::AttachedToCall => {
                        Promise::new(consumer_id.clone()).transfer(*amount);
                    }
                }
            }
            PrepaidFee::FungibleToken {
                token: _,
                amount: _,
                payment_type: _,
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
                payment_type: _,
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
                payment_type: _,
            } => unimplemented!(),
        }
    }
}

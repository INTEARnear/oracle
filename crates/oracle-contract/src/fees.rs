use near_sdk::{env, json_types::U128, near, store::LookupMap, Gas, NearToken, Promise};
use near_sdk_contract_tools::ft::ext_nep141;

use crate::{
    balance::FtId,
    consumer::{ConsumerId, OracleEvent},
    producer::{Producer, ProducerId},
};
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
        payment_type: NearPaymentType,
    },
    FungibleToken {
        token: FtId,
        amount: U128,
        payment_type: FtPaymentType,
    },
}

#[near(serializers=[json, borsh])]
#[derive(Debug, Clone, PartialEq)]
pub enum NearPaymentType {
    ForSpecificProducer,
    ForAllProducers,
    AttachedToCall,
}

#[near(serializers=[json, borsh])]
#[derive(Debug, Clone, PartialEq)]
pub enum FtPaymentType {
    ForSpecificProducer,
    ForAllProducers,
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
        let producer = self.producers.get(&env::predecessor_account_id()).unwrap();
        OracleEvent::ProducerCreated(Producer {
            account_id: producer.account_id.clone(),
            requests_succeded: producer.requests_succeded,
            requests_timed_out: producer.requests_timed_out,
            requests_pending: LookupMap::new(b"dontcare".as_slice()),
            fee: producer.fee.clone(),
            send_callback: producer.send_callback,
            name: producer.name.clone(),
            description: producer.description.clone(),
            example_input: producer.example_input.clone(),
        })
        .emit();
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
                        payment_type: NearPaymentType::AttachedToCall,
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
                            payment_type: NearPaymentType::ForSpecificProducer,
                        });
                    }
                }

                if consumer.near_balance >= *prepaid_amount {
                    consumer.near_balance =
                        consumer.near_balance.checked_sub(*prepaid_amount).unwrap();
                    return Some(PrepaidFee::Near {
                        amount: *prepaid_amount,
                        payment_type: NearPaymentType::ForAllProducers,
                    });
                }

                None
            }
            ProducerFee::FungibleToken {
                token,
                prepaid_amount,
            } => {
                let consumer = self
                    .consumers
                    .get_mut(consumer_id)
                    .expect("Consumer is not registered");

                if let Some(ft_balance) = consumer
                    .ft_balances_producer
                    .get_mut(&(producer_id.clone(), token.clone()))
                {
                    if *ft_balance >= *prepaid_amount {
                        *ft_balance = ft_balance.0.checked_sub(prepaid_amount.0).unwrap().into();
                        if ft_balance.0 == 0 {
                            consumer
                                .ft_balances_producer
                                .remove(&(producer_id.clone(), token.clone()));
                        }
                        return Some(PrepaidFee::FungibleToken {
                            token: token.clone(),
                            amount: *prepaid_amount,
                            payment_type: FtPaymentType::ForSpecificProducer,
                        });
                    }
                }

                if let Some(ft_balance) = consumer.ft_balances.get_mut(token) {
                    if *ft_balance >= *prepaid_amount {
                        *ft_balance = ft_balance.0.checked_sub(prepaid_amount.0).unwrap().into();
                        if ft_balance.0 == 0 {
                            consumer.ft_balances.remove(token);
                        }
                        return Some(PrepaidFee::FungibleToken {
                            token: token.clone(),
                            amount: *prepaid_amount,
                            payment_type: FtPaymentType::ForAllProducers,
                        });
                    }
                }

                None
            }
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
                NearPaymentType::ForSpecificProducer => {
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
                NearPaymentType::ForAllProducers => {
                    let consumer = self
                        .consumers
                        .get_mut(consumer_id)
                        .expect("Consumer is not registered");

                    consumer.near_balance = consumer
                        .near_balance
                        .checked_add(NearToken::from_yoctonear(refund_amount.0))
                        .unwrap();
                }
                NearPaymentType::AttachedToCall => {
                    Promise::new(consumer_id.clone())
                        .transfer(NearToken::from_yoctonear(refund_amount.0));
                }
            },
            PrepaidFee::FungibleToken {
                token,
                amount: _,
                payment_type,
            } => match payment_type {
                FtPaymentType::ForSpecificProducer => {
                    let consumer = self
                        .consumers
                        .get_mut(consumer_id)
                        .expect("Consumer is not registered");

                    if let Some(ft_balance) = consumer
                        .ft_balances_producer
                        .get_mut(&(producer_id.clone(), token.clone()))
                    {
                        *ft_balance = ft_balance.0.checked_add(refund_amount.0).unwrap().into();
                    } else {
                        consumer
                            .ft_balances_producer
                            .insert((producer_id.clone(), token.clone()), refund_amount);
                    }
                }
                FtPaymentType::ForAllProducers => {
                    let consumer = self
                        .consumers
                        .get_mut(consumer_id)
                        .expect("Consumer is not registered");

                    if let Some(ft_balance) = consumer.ft_balances.get_mut(token) {
                        *ft_balance = ft_balance.0.checked_add(refund_amount.0).unwrap().into();
                    } else {
                        consumer.ft_balances.insert(token.clone(), refund_amount);
                    }
                }
            },
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
            } => match payment_type {
                NearPaymentType::ForSpecificProducer => {
                    let consumer = self
                        .consumers
                        .get_mut(consumer_id)
                        .expect("Consumer is not registered");

                    if let Some(near_balance) = consumer.near_balance_producer.get_mut(producer_id)
                    {
                        *near_balance = near_balance.checked_add(*amount).unwrap();
                    } else {
                        consumer
                            .near_balance_producer
                            .insert(producer_id.clone(), *amount);
                    }
                }
                NearPaymentType::ForAllProducers => {
                    let consumer = self
                        .consumers
                        .get_mut(consumer_id)
                        .expect("Consumer is not registered");

                    consumer.near_balance = consumer.near_balance.checked_add(*amount).unwrap();
                }
                NearPaymentType::AttachedToCall => {
                    Promise::new(consumer_id.clone()).transfer(*amount);
                }
            },
            PrepaidFee::FungibleToken {
                token,
                amount,
                payment_type,
            } => match payment_type {
                FtPaymentType::ForSpecificProducer => {
                    let consumer = self
                        .consumers
                        .get_mut(consumer_id)
                        .expect("Consumer is not registered");

                    if let Some(ft_balance) = consumer
                        .ft_balances_producer
                        .get_mut(&(producer_id.clone(), token.clone()))
                    {
                        *ft_balance = ft_balance.0.checked_add(amount.0).unwrap().into();
                    } else {
                        consumer
                            .ft_balances_producer
                            .insert((producer_id.clone(), token.clone()), *amount);
                    }
                }
                FtPaymentType::ForAllProducers => {
                    let consumer = self
                        .consumers
                        .get_mut(consumer_id)
                        .expect("Consumer is not registered");

                    if let Some(ft_balance) = consumer.ft_balances.get_mut(token) {
                        *ft_balance = ft_balance.0.checked_add(amount.0).unwrap().into();
                    } else {
                        consumer.ft_balances.insert(token.clone(), *amount);
                    }
                }
            },
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
                token,
                amount,
                payment_type: _,
            } => {
                if let Some(deposit_amount) =
                    amount.0.checked_sub(refund_amount.unwrap_or(U128(0)).0)
                {
                    // TODO handle when the token is not registered in producer's account
                    ext_nep141::ext(token.clone())
                        .with_static_gas(Gas::from_tgas(10))
                        .with_attached_deposit(NearToken::from_yoctonear(1))
                        .ft_transfer(producer_id.clone(), deposit_amount.into(), None);
                } else {
                    env::panic_str("Refund amount is greater than prepaid amount")
                }
            }
        }
    }
}

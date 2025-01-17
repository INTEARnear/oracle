use near_sdk::{
    env, json_types::U128, log, near, serde_json, AccountId, Gas, NearToken, Promise,
    PromiseOrValue,
};
use near_sdk_contract_tools::ft::{ext_nep141, Nep141Receiver};

use crate::producer::ProducerId;
#[cfg(feature = "contract")]
use crate::{Oracle, OracleExt};

pub type FtId = AccountId;

#[cfg(feature = "contract")]
#[near]
impl Oracle {
    #[payable]
    pub fn deposit_near(&mut self, account_id: Option<AccountId>, producer_id: Option<ProducerId>) {
        let amount = env::attached_deposit();
        let account_id = account_id.unwrap_or_else(env::predecessor_account_id);
        let consumer = self
            .consumers
            .get_mut(&account_id)
            .expect("Consumer is not registered");
        if let Some(producer_id) = producer_id {
            near_sdk::require!(
                self.producers.contains_key(&producer_id),
                "Producer doesn't exist"
            );
            if let Some(near_balance) = consumer.near_balance_producer.get_mut(&producer_id) {
                *near_balance = near_balance.checked_add(amount).expect("Overflow");
            } else {
                consumer
                    .near_balance_producer
                    .insert(producer_id.clone(), amount);
            }
            log!("Deposited {amount} to {account_id} for {producer_id}",);
        } else {
            consumer.near_balance = consumer.near_balance.checked_add(amount).expect("Overflow");
            log!("Deposited {amount} to {account_id}",);
        }
    }

    pub fn get_deposit_near(
        &self,
        account_id: AccountId,
        producer_id: Option<ProducerId>,
    ) -> NearToken {
        let consumer = self
            .consumers
            .get(&account_id)
            .expect("Consumer is not registered");
        if let Some(producer_id) = producer_id {
            consumer
                .near_balance_producer
                .get(&producer_id)
                .cloned()
                .unwrap_or(NearToken::from_near(0))
        } else {
            consumer.near_balance
        }
    }

    pub fn get_deposit_ft(
        &self,
        account_id: AccountId,
        producer_id: Option<ProducerId>,
        ft_id: FtId,
    ) -> U128 {
        let consumer = self
            .consumers
            .get(&account_id)
            .expect("Consumer is not registered");
        if let Some(producer_id) = producer_id {
            consumer
                .ft_balances_producer
                .get(&(producer_id, ft_id))
                .cloned()
                .unwrap_or(U128(0))
        } else {
            consumer.ft_balances.get(&ft_id).cloned().unwrap_or(U128(0))
        }
    }

    #[payable]
    pub fn withdraw_near(&mut self, amount: NearToken, producer_id: Option<ProducerId>) {
        let account_id = env::predecessor_account_id();
        let consumer = self
            .consumers
            .get_mut(&account_id)
            .expect("Consumer is not registered");
        if let Some(producer_id) = producer_id {
            near_sdk::require!(
                self.producers.contains_key(&producer_id),
                "Producer doesn't exist"
            );
            let near_balance = consumer
                .near_balance_producer
                .get_mut(&producer_id)
                .expect("No balance for producer");
            near_sdk::require!(*near_balance >= amount, "Not enough balance for producer");
            *near_balance = near_balance.checked_sub(amount).expect("Underflow");
            if near_balance.is_zero() {
                consumer.near_balance_producer.remove(&producer_id);
            }
            log!("Withdrew {amount} from {account_id} for {producer_id}",);
        } else {
            near_sdk::require!(consumer.near_balance >= amount, "Not enough balance");
            consumer.near_balance = consumer
                .near_balance
                .checked_sub(amount)
                .expect("Underflow");
            log!("Withdrew {amount} from {account_id}",);
        }
        Promise::new(account_id).transfer(amount);
    }

    #[payable]
    pub fn withdraw_ft(&mut self, amount: U128, producer_id: Option<ProducerId>, ft_id: FtId) {
        let account_id = env::predecessor_account_id();
        let consumer = self
            .consumers
            .get_mut(&account_id)
            .expect("Consumer is not registered");
        if let Some(producer_id) = producer_id {
            near_sdk::require!(
                self.producers.contains_key(&producer_id),
                "Producer doesn't exist"
            );
            let ft_balance = consumer
                .ft_balances_producer
                .get_mut(&(producer_id.clone(), ft_id.clone()))
                .expect("No balance for producer");
            near_sdk::require!(*ft_balance >= amount, "Not enough balance for producer");
            *ft_balance = ft_balance
                .0
                .checked_sub(amount.0)
                .expect("Underflow")
                .into();
            if *ft_balance == 0.into() {
                consumer
                    .ft_balances_producer
                    .remove(&(producer_id.clone(), ft_id.clone()));
            }
            log!(
                "Withdrew {amount} {ft_id} from {account_id} for {producer_id}",
                amount = amount.0
            );
        } else {
            let ft_balance = consumer.ft_balances.get_mut(&ft_id).expect("No balance");
            near_sdk::require!(*ft_balance >= amount, "Not enough balance");
            *ft_balance = ft_balance
                .0
                .checked_sub(amount.0)
                .expect("Underflow")
                .into();
            if *ft_balance == 0.into() {
                consumer.ft_balances.remove(&ft_id);
            }
            log!(
                "Withdrew {amount} {ft_id} from {account_id}",
                amount = amount.0
            );
        }

        ext_nep141::ext(ft_id)
            .with_static_gas(Gas::from_tgas(10))
            .with_attached_deposit(NearToken::from_yoctonear(1))
            .ft_transfer(account_id, amount, None);
    }
}

#[near(serializers=[json])]
struct FtDepositArgs {
    pub account_id: Option<AccountId>,
    pub producer_id: Option<ProducerId>,
}

#[cfg(feature = "contract")]
#[near]
impl Nep141Receiver for Oracle {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let args = serde_json::from_str::<FtDepositArgs>(&msg).expect("Invalid msg");
        let account_id = args.account_id.unwrap_or(sender_id);
        let ft_id = env::predecessor_account_id();
        let consumer = self
            .consumers
            .get_mut(&account_id)
            .expect("Consumer is not registered");
        if let Some(producer_id) = args.producer_id {
            near_sdk::require!(
                self.producers.contains_key(&producer_id),
                "Producer doesn't exist"
            );
            if let Some(ft_balance) = consumer
                .ft_balances_producer
                .get_mut(&(producer_id.clone(), ft_id.clone()))
            {
                *ft_balance = ft_balance.0.checked_add(amount.0).expect("Overflow").into();
            } else {
                consumer
                    .ft_balances_producer
                    .insert((producer_id.clone(), ft_id.clone()), amount);
            }
            log!(
                "Deposited {amount} {ft_id} to {account_id} for {producer_id}",
                amount = amount.0
            );
        } else {
            if let Some(ft_balance) = consumer.ft_balances.get_mut(&ft_id) {
                *ft_balance = ft_balance.0.checked_add(amount.0).expect("Overflow").into();
            } else {
                consumer.ft_balances.insert(ft_id.clone(), amount);
            }
            log!(
                "Deposited {amount} {ft_id} to {account_id}",
                amount = amount.0
            );
        }
        PromiseOrValue::Value(U128(0))
    }
}

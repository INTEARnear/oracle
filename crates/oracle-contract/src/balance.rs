use near_sdk::{env, json_types::U128, log, near, serde_json, AccountId, PromiseOrValue};

use crate::{producer::ProducerId, Contract, ContractExt};

pub type FtId = AccountId;

#[near(serializers=[json])]
struct FtDepositArgs {
    pub account_id: Option<AccountId>,
    pub producer_id: Option<ProducerId>,
}

// TODO: Can't use git near-sdk with near-sdk-contract-tools
//
// impl Nep141Receiver for Contract {
//     fn ft_on_transfer(
//         &mut self,
//         sender_id: AccountId,
//         amount: U128,
//         msg: String,
//     ) -> PromiseOrValue<U128> {
//
//     }
// }
impl Contract {
    pub fn ft_on_transfer(
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
                    .insert((producer_id, ft_id), amount);
            }
            log!("Deposited {amount} {account_id} to {sender_id} for {producer_id}");
        } else {
            if let Some(ft_balance) = consumer.ft_balances.get_mut(&ft_id) {
                *ft_balance = ft_balance.0.checked_add(amount.0).expect("Overflow").into();
            } else {
                consumer.ft_balances.insert(ft_id, amount);
            }
            log!("Deposited {amount} {account_id} to {sender_id}");
        }
        PromiseOrValue::Value(U128(0))
    }
}

#[near]
impl Contract {
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
                consumer.near_balance_producer.insert(producer_id, amount);
            }
            log!("Deposited {amount} NEAR to {account_id} for {producer_id}");
        } else {
            consumer.near_balance = consumer.near_balance.checked_add(amount).expect("Overflow");
            log!("Deposited {amount} NEAR to {account_id}");
        }
    }

    // TODO: Fee balance getters
}

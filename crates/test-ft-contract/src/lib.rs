use near_sdk::{json_types::U128, near, AccountId};
use near_sdk_contract_tools::ft::*;

#[derive(Default, FungibleToken)]
#[near(contract_state)]
pub struct MyFtContract {}

#[near]
impl MyFtContract {
    #[init]
    pub fn new() -> Self {
        let mut contract = Self {};

        contract.set_metadata(&ContractMetadata::new("My Fungible Token", "MYFT", 24));

        contract
    }

    pub fn mint(&mut self, account_id: AccountId, amount: U128) {
        Nep141Controller::mint(
            self,
            &Nep141Mint {
                receiver_id: account_id.into(),
                amount: amount.0,
                memo: None,
            },
        )
        .unwrap();
    }
}

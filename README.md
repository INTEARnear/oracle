# Oracle

INTEAR Oracle is a project that aims to bring off-chain computation and data availability into NEAR, with the best developer experience.

It utilizes [Yielded execution](https://github.com/near/NEPs/pull/519) and [inindexer](https://github.com/INTEARnear/inindexer) for receiving and answering requests.

> Try it out!

1. Register yourself:
`near contract call-function as-transaction dev-unaudited-v0.oracle.intear.near register_consumer json-args '{"account_id":"<account.near>"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as <account.near> network-config mainnet sign-with-keychain send`
2. Deposit 0.01N to your fee balance (can deposit 0.1N for 10 requests):
`near contract call-function as-transaction dev-unaudited-v0.oracle.intear.near register_consumer json-args '{"account_id":"<account.near>"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as <account.near> network-config mainnet sign-with-keychain send`
2. Request
`near contract call-function as-transaction dev-unaudited-v0.oracle.intear.near request json-args '{"producer_id":"gpt4o.oracle.intear.near","request_data":"Hello, GPT-4o!"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as <account.near> network-config mainnet sign-with-keychain send`

## Contract

### Setting up

- To set up a producer that submits data on chain, use `add_producer(account_id: AccountId)` method.
- To set up a consumer that requests data from producers, use `register_consumer(account_id: AccountId)` method, as well as `set_fee`.

Don't forget about storage deposits.

### Usage

There are just 2 interesting methods: `request` and `respond`, which are meant for **Consumers** and **Producers**.

1. Request: Specify `producer_id: AccountId` and `request_data: String` parameters. This method returns a `Promise` that you can use to chain
   calls and use with `#[callback_result]` or `#[callback_unwrap]` as any other promise. The return type is `Option<Response>`.
2. If the consumer has enough deposit for fees, they get frozen in the contract for this specific request, and a log is fired:
   `EVENT_JSON:{"standard":"intear-oracle","version":"1.0.0","event":"request","data":{"consumer_id":"{consumer}","request_id":"0","request_data":"Hello World!"}}`.
   The contract stores a **resumption token**, the producer can choose to subsidize the storage deposit, it gets deleted soon anyway. 
3. The indexer in [crates/indexer](crates/indexer) catches all events with `standard: "intear-oracle"` and `event: "request"`, gets the data,
   and sends it using `respond(request_id: StringifiedNumber, response: Response)` where Response is defined in [crates/oracle-contract/src/producer.rs](crates/oracle-contract/src/producer.rs).
4. The contract removes the pending request freeing up the storage, and resumes the yielded execution with Some(response).
5. If the indexer fails to respond in 200 blocks (`yield_timeout_length_in_blocks` NEAR parameter config), the yielded execution resumes with None response.

### Paying for usage

Some data producers may choose to charge a fee for requesting some data using this method:

`set_fee(fee: ProducerFee)`, ProducerFee is defined in [crates/oracle-contract/src/producer.rs](crates/oracle-contract/src/producer.rs).

The oracle contract uses prepaid balance to optimize the amount of receipts generated, reducing the latency, but may allow per-request pricing in
the future, if someone wants this feature added. To top up your balance, use these methods:

`deposit_near(account_id: Option<AccountId> default predecessor, producer_id: Option<AccountId> default all)`
`ft_transfer_call` on the token with msg `account_id: Option<AccountId> default predecessor, producer_id: Option<AccountId> default all`

Parameters:
- `account_id`: Account to top up. If not specified, defaults to the predecessor account, but you can top up someone else's balance in the contract
  if you specify it
- `producer_id`: Top up balance for a specific producer. You can choose to not specify that parameter, and your *general balance* will be increased,
  which allows you to use the same balance for all producers. Depending on how you want to use it, you may want to go with separate balances.

### Callbacks

If your contract can submit data without off-chain intervention, or needs to store requests - you can call `set_send_callback(send_callback: bool)`,
and if this value is set to `true`, the oracle contract will call `on_request(request_id: String, request_data: String, charged_fee: PrepaidFee)` on
the producer contract every time it receives a request.

### Minimum fee

Producers can set up a minimum upfront fee - if you don't have enough on your balance / attached, the oracle won't be called.

Producers can choose to refund a part of the fee, for example, if it's based on OpenAI tokens usage. It's hard to calculate spendings beforehand, so
it's easier to just charge $1 upfront for each query and refund $0.9995 in the `respond` call.

### Withdrawing balance

To withdraw your fees as a consumer, use `withdraw_near()` or `withdraw_ft(token_id: AccountId)` methods. Producers get their fees automatically once
they fulfill a request.

### View methods and third-party standard method

Check out our nearblocks page with all other methods. Tl;dr: You can check deposit balances, stats of successful / timed out requests per producer, and
storage management methods.

## Build & Test

Currently, it's quite hard to test, because yielded execution is only included in a release candidate, and some dependencies haven't been updated
for months, or may be incompatible with the git near-sdk (such as near-sdk-contract-tools). That's also the reason the contract has not fully implemented
all standards and best security practices yet. To test it, you also need to compile (or download, if it's already available by the time you read this)
`neard` in sandbox mode, and set `NEAR_SANDBOX_BIN_PATH` environment variable.

## Use cases

This contract is designed to simplify the delivery of off-chain data and create a data marketplace. Here are some situations where intear oracle can be useful:

- Offloading heavy computations
- Generative AI
- HTTP API requests
- Randomness (it's possible to do a more random and more verifiable but more centralized than `near_sdk::env::random_seed`)
- Price data. You can produce data using smart contracts if you want a more decentralized and fault-tolerant source of data
- MPC signatures
- Inscriptions
- Centralized bridges
- Data from other blockchains (can be used in chain abstracted dapps to verify that transaction was sent and / or get tx result)
- Anything else, it's basically a set of building blocks that give you an ability to charge a fee for a request. All you need to do is set up the producer
  account and send these `respond` transactions.

# Bitcoin Oracle Indexer

An indexer service that monitors the Bitcoin Oracle contract on NEAR Protocol and processes relevant events.

## Environment Variables

- `ACCOUNT_ID`: The account ID of the oracle producer
- `CONTRACT_ID`: The contract ID of the Bitcoin Oracle contract (not to confuse with the Intear Oracle main contract)
- `PRIVATE_KEY`: The private key for the oracle producer account
- `BITCOIN_RPC_URL`: URL of the Bitcoin node RPC endpoint

## Running

```bash
cargo run
``` 
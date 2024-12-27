# Bitcoin Oracle Indexer

An indexer service that monitors the Bitcoin Oracle contract on NEAR Protocol and processes relevant events.

## Environment Variables

- `ACCOUNT_ID`: The account ID of the oracle producer
- `PRIVATE_KEY`: The private key for the oracle producer account
- `BITCOIN_RPC_URL`: URL of the Bitcoin node RPC endpoint
- `BITCOIN_RPC_AUTH`: Authentication for the Bitcoin RPC (format: username:password)

## Running

```bash
cargo run
``` 
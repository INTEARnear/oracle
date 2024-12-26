# Reclaim GPT indexer

An indexer for OpenAI requests that returns responses with a proof by Reclaim Protocol. Reclaim zkfetch requires the output to be consistent, but OpenAI is not consistent even with `seed` parameter, so it'll only work with short responses that don't involve any creative thinking ("write a 1000-word story about a slime" won't work, but "is 9.9 greater than 9.11? respond with just Yes or No without punctuation, just a single word" probably will). Also, small models (`gpt-3.5-turbo`, `gpt-4o-mini`) work better.

Environment variables:
- ACCOUNT_ID (your-account.near)
- PRIVATE_KEY (ed25519:your-account)
- CONTRACT_ID (gpt-reclaim.oracle.intear.near)
- ORACLE_CONTRACT_ID (dev-unaudited-v1.oracle.intear.near)
- APP_ID (Reclaim app id)
- APP_SECRET (Reclaim app secret)
- OPENAI_API_KEY (sk-your-api-key)
- MAINNET (true or empty)

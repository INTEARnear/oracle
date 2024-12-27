# Bitcoin Oracle Contract

A NEAR smart contract that serves as an oracle for Bitcoin transaction data. This contract can verify Bitcoin transaction inclusion. Anyone can submit a proof, it will be verified on-chain using https://github.com/Near-One/btc-light-client-contract and returned to the requester.

## Features

- Bitcoin transaction inclusion verification
- Return number of confirmations (decreased by 1 because of light client delay)
- Possible in the future: Include rich information about the transaction (e.g. sender, receiver, amounts, etc.)

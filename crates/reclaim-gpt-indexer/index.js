import dotenv from 'dotenv';
import nearApi from 'near-api-js';
import fs from 'node:fs/promises';
import { ReclaimClient } from "@reclaimprotocol/zk-fetch";
dotenv.config();
const { KeyPair, keyStores, connect, transactions } = nearApi;

const OPERATOR_ACCOUNT_ID = process.env.ACCOUNT_ID;
const OPERATOR_PRIVATE_KEY = process.env.PRIVATE_KEY;
const PRODUCER_CONTRACT_ID = process.env.CONTRACT_ID;
const ORACLE_CONTRACT_ID = process.env.ORACLE_CONTRACT_ID;
const APP_ID = process.env.APP_ID ?? "0xF218B59D7794e32693f5D3236e011C233E249105";
const APP_SECRET = process.env.APP_SECRET ?? "0xe7cc556f58d92618e04ebbd16744be753eb4d06d569590df341c89e25f6ecc9c";
const OPENAI_API_KEY = process.env.OPENAI_API_KEY;

const keyStore = new keyStores.InMemoryKeyStore();
const config = {
    keyStore,
    networkId: "testnet",
    nodeUrl: "https://rpc.testnet.near.org",
};
const near = await connect({ ...config, keyStore })
await keyStore.setKey("testnet", OPERATOR_ACCOUNT_ID, KeyPair.fromString(OPERATOR_PRIVATE_KEY));
const account = await near.account(OPERATOR_ACCOUNT_ID);

let currentBlock = await fs.readFile("last-processed-block.txt", "utf-8").catch(() => "42376888");
let attempt = 0;
async function sleep(ms) {
    return new Promise((resolve, _reject) => {
        setTimeout(() => resolve(), ms);
    });
}
while (true) {
    try {
        await fetch(`https://testnet.neardata.xyz/v0/block/${currentBlock}`, {
            headers: {
                "User-Agent": `Intear Reclaim oracle indexer`
            },
        })
            .then(res => res.json())
            .then(async (res) => {
                try {
                    if (!res) {
                        console.log("Skipping block", currentBlock);
                        currentBlock++;
                        return;
                    }
                    console.log("Processing block", res.block.header.height);
                    for (const shard of res.shards) {
                        for (const receipt of shard.receipt_execution_outcomes) {
                            if (receipt.execution_outcome.outcome.status.SuccessValue || receipt.execution_outcome.outcome.status.SuccessReceiptId) {
                                if (receipt.receipt.receiver_id === ORACLE_CONTRACT_ID) {
                                    for (const log of receipt.execution_outcome.outcome.logs) {
                                        if (log.startsWith("EVENT_JSON:")) {
                                            const event = JSON.parse(log.substring("EVENT_JSON:".length));
                                            if (event.standard === "intear-oracle" && event.event === "request") {
                                                const data = event.data;
                                                if (data.producer_id === PRODUCER_CONTRACT_ID) {
                                                    (async function () {
                                                        for (let responseAttempt = 0; responseAttempt < 5; responseAttempt++) {
                                                            console.log(`Processing request ${data.request_id}, attempt ${responseAttempt}`);
                                                            try {
                                                                const requestData = JSON.parse(data.request_data);
                                                                const url = "https://api.openai.com/v1/chat/completions";
                                                                const publicOptions = {
                                                                    "method": "POST",
                                                                    "body": JSON.stringify({
                                                                        "model": requestData.model,
                                                                        "messages": [
                                                                            {
                                                                                "role": "system",
                                                                                "content": requestData.system_message,
                                                                            },
                                                                            {
                                                                                "role": "user",
                                                                                "content": requestData.user_message,
                                                                            },
                                                                        ],
                                                                        "seed": requestData.seed,
                                                                    }),
                                                                    "headers": {
                                                                        "Content-Type": "application/json"
                                                                    },
                                                                };
                                                                const privateOptions = {
                                                                    headers: { "Authorization": `Bearer ${OPENAI_API_KEY}` },
                                                                };
                                                                const response = await fetch(url, { ...publicOptions, headers: { ...privateOptions.headers, ...publicOptions.headers } }).then(res => res.text());
                                                                Object.assign(privateOptions, { responseMatches: [{ type: "contains", value: response.replace(/(.|\n)+"created": \d+,/, "") }] });
                                                                let proof = await getProof(url, publicOptions, privateOptions, 1, 0);
                                                                if (proof) {
                                                                    console.log("Proof created");
                                                                    console.log(`Proof: ${JSON.stringify(proof)}`);
                                                                } else {
                                                                    console.error("Failed to create a proof")
                                                                    console.error("Response:", response);
                                                                    throw new Error("Failed to create a proof")
                                                                }
                                                                await account.signAndSendTransaction({
                                                                    receiverId: PRODUCER_CONTRACT_ID,
                                                                    actions: [
                                                                        transactions.functionCall("submit", Buffer.from(JSON.stringify({
                                                                            request_id: data.request_id,
                                                                            proof,
                                                                        })), 300_000_000_000_000, "0")
                                                                    ],
                                                                });
                                                                break;
                                                            } catch (e) {
                                                                console.error(`Failed to process response ${responseAttempt} attempt: ${e}`);
                                                                await sleep(500);
                                                            }
                                                        }
                                                    })();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } catch (e) {
                    console.error(`Failed to process block ${currentBlock}: ${e}`);
                    console.error(e)
                }
                currentBlock++;
                attempt = 0;
                await fs.writeFile("last-processed-block.txt", currentBlock.toString());
            });
    } catch (e) {
        console.error(`Failed to fetch block ${currentBlock} attempt ${attempt}: ${e}`);
        attempt++;
        await sleep(1000);
    }
}

export async function getProof(url, publicOptions, privateOptions, retries, retryInterval, extractData) {
    const reclaim = new ReclaimClient(
        APP_ID,
        APP_SECRET,
    );

    let proof;
    try {
        proof = await reclaim.zkFetch(url, publicOptions, privateOptions, retries, retryInterval, extractData);
    } catch (error) {
        console.error(`Error fetching response: ${error}`);
        return null;
    }
    console.log("PROOF", proof);

    try {
        if (!await ReclaimClient.verifySignedProof(proof)) {
            console.warn("Proof is invalid");
            return null;
        }
    } catch (error) {
        console.error(`Error validating proof: ${error}`);
        return null;
    }

    const proofData = {
        claimInfo: {
            provider: proof.claim.provider,
            parameters: proof.claim.parameters,
            context: proof.claim.context,
        },
        signedClaim: {
            claim: {
                identifier: proof.claim.identifier.replace(/^0x/, ''),
                owner: proof.claim.owner.replace(/^0x/, ''),
                epoch: proof.claim.epoch,
                timestampS: proof.claim.timestampS,
            },
            signatures: [proof.signatures.claimSignature.toString("hex")],
        },
    };

    return proofData;
}

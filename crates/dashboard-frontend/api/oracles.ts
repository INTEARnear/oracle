import { EventStreamClient } from "@intear/inevents-websocket-client";

export const ORACLE_CONTRACT_ID = "dev-unaudited-v1.oracle.intear.near";

export interface Oracle {
    id: string;
    name: string;
    description: string;
    successes: number;
    failures: number;
    fee: {
        amount: string;
        token: string;
    };
    example_input?: string;
}

export async function fetchOracles(): Promise<Oracle[]> {
    try {
        const response = await fetch('http://localhost:9000/oracles');
        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        return await response.json();
    } catch (error) {
        console.error('Error fetching oracles:', error);
        return [];
    }
}

interface LogNep297Event<T> {
    block_height: number;
    block_timestamp_nanosec: string;
    transaction_id: string;
    receipt_id: string;
    account_id: string;
    predecessor_id: string;
    event_standard: string;
    event_version: string;
    event_event: string;
    event_data: T;
}

interface Producer {
    account_id: string;
    requests_succeded: number;
    requests_timed_out: number;
    fee: ProducerFee;
    send_callback: boolean;
    name: string;
    description: string;
    example_input?: string;
}

type ProducerFee =
    | "None"
    | { Near: { prepaid_amount: string } }
    | { FungibleToken: { token: string, prepaid_amount: string } };

export async function listenForUpdates(onOracleUpdate: (oracle: Oracle) => void) {
    const client = EventStreamClient.default();
    await client.streamEvents<LogNep297Event<Producer>>("log_nep297", {
        And: [
            {
                path: "account_id",
                operator: {
                    Equals: ORACLE_CONTRACT_ID
                }
            },
            {
                path: "event_standard",
                operator: {
                    Equals: "intear-oracle"
                }
            },
            {
                path: ".",
                operator: {
                    Or: [
                        {
                            path: "event_event",
                            operator: {
                                Equals: "producer_created"
                            }
                        },
                        {
                            path: "event_event",
                            operator: {
                                Equals: "producer_updated"
                            }
                        }
                    ]
                }
            },
        ]
    }, async (event) => {
        let producer: Producer = event.event_data;
        let oracle: Oracle = {
            id: producer.account_id,
            name: producer.name,
            description: producer.description,
            successes: producer.requests_succeded,
            failures: producer.requests_timed_out,
            example_input: producer.example_input,
            fee: (() => {
                if (producer.fee === "None") {
                    return {
                        amount: "0",
                        token: "near"
                    };
                }
                if ('Near' in producer.fee) {
                    return {
                        amount: producer.fee.Near.prepaid_amount,
                        token: "near"
                    };
                }
                if ('FungibleToken' in producer.fee) {
                    return {
                        amount: producer.fee.FungibleToken.prepaid_amount,
                        token: producer.fee.FungibleToken.token
                    };
                }
                throw new Error('Unknown fee type');
            })()
        }
        onOracleUpdate(oracle);
    });
}

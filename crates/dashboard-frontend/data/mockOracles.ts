export interface Oracle {
    id: number;
    name: string;
    description: string;
    successes: number;
    failures: number;
    fee: {
        amount: string;
        token: string;
    };
    exampleInput?: string;
}

export const mockOracles: Oracle[] = [
    {
        id: 1,
        name: "ChainLink Price Feed",
        description: "Real-time cryptocurrency price data aggregated from multiple sources",
        successes: 1200,
        failures: 96,
        fee: {
            amount: "1000000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "token.0xshitzu.near",
    },
    {
        id: 2,
        name: "Weather Oracle",
        description: "Global weather data from multiple meteorological stations",
        successes: 50000,
        failures: 2,
        fee: {
            amount: "100000000000000000000000",
            token: "near"
        },
    }
]; 
export interface Oracle {
  id: number;
  name: string;
  description: string;
  users: number;
  successRate: number;
  failureRate: number;
  fee: {
    amount: number;
    token: string;
  };
}

export const mockOracles: Oracle[] = [
  {
    id: 1,
    name: "ChainLink Price Feed",
    description: "Real-time cryptocurrency price data aggregated from multiple sources",
    users: 50000,
    successRate: 99.9,
    failureRate: 0.1,
    fee: {
      amount: 0.1,
      token: "LINK"
    }
  },
  {
    id: 2,
    name: "Weather Oracle",
    description: "Global weather data from multiple meteorological stations",
    users: 25000,
    successRate: 99.5,
    failureRate: 0.5,
    fee: {
      amount: 0.05,
      token: "LINK"
    }
  }
  // ... Add more oracles as needed
]; 
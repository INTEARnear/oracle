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
    exampleInput?: string;
}

export const mockOracles: Oracle[] = [
    {
        id: "chainlink-price-feed.near",
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
        id: "weather-oracle.near",
        name: "Weather Oracle",
        description: "Global weather data from multiple meteorological stations",
        successes: 50000,
        failures: 2,
        fee: {
            amount: "100000000000000000000000",
            token: "near"
        },
    },
    {
        id: "stock-market-feed.near",
        name: "Stock Market Data Feed",
        description: "Real-time stock market data from major global exchanges",
        successes: 890000,
        failures: 123,
        fee: {
            amount: "5000000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "AAPL",
    },
    {
        id: "sports-results.near",
        name: "Sports Results Oracle",
        description: "Live sports scores and match results from major leagues worldwide",
        successes: 25000,
        failures: 42,
        fee: {
            amount: "2500000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "NBA_GAME_20230615_GSW_LAL",
    },
    {
        id: "flight-status.near",
        name: "Flight Status Tracker",
        description: "Real-time flight status and delay information from global airports",
        successes: 150000,
        failures: 89,
        fee: {
            amount: "3000000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "AA1234",
    },
    {
        id: "random-number.near",
        name: "Random Number Generator",
        description: "Verifiable random number generation for gaming and lottery applications",
        successes: 1000000,
        failures: 0,
        fee: {
            amount: "1000000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "range:1-1000",
    },
    {
        id: "identity-verify.near",
        name: "Identity Verification",
        description: "KYC and identity verification service integration",
        successes: 75000,
        failures: 150,
        fee: {
            amount: "10000000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "user_id:12345",
    },
    {
        id: "carbon-credits.near",
        name: "Carbon Credit Tracker",
        description: "Real-time carbon credit pricing and verification",
        successes: 45000,
        failures: 23,
        fee: {
            amount: "200000000000000000000000",
            token: "near"
        },
        exampleInput: "credit_id:VCS-123",
    },
    {
        id: "iot-sensors.near",
        name: "IoT Sensor Network",
        description: "Distributed IoT sensor data aggregation and verification",
        successes: 500000,
        failures: 1200,
        fee: {
            amount: "1500000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "sensor_id:456",
    },
    {
        id: "dns-validator.near",
        name: "DNS Record Validator",
        description: "Domain name system record verification and resolution",
        successes: 250000,
        failures: 45,
        fee: {
            amount: "1000000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "example.com",
    },
    {
        id: "social-metrics.near",
        name: "Social Media Metrics",
        description: "Real-time social media engagement and follower metrics",
        successes: 180000,
        failures: 234,
        fee: {
            amount: "2000000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "@username",
    },
    {
        id: "nft-floor-price.near",
        name: "NFT Floor Price",
        description: "Real-time NFT collection floor prices across major marketplaces",
        successes: 350000,
        failures: 89,
        fee: {
            amount: "1500000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "collection:boredapes",
    },
    {
        id: "earthquake-monitor.near",
        name: "Earthquake Monitor",
        description: "Global seismic activity monitoring and reporting",
        successes: 15000,
        failures: 2,
        fee: {
            amount: "150000000000000000000000",
            token: "near"
        },
        exampleInput: "region:pacific-rim",
    },
    {
        id: "election-results.near",
        name: "Election Results",
        description: "Official election results verification and reporting",
        successes: 5000,
        failures: 0,
        fee: {
            amount: "5000000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "election_id:2024_us",
    },
    {
        id: "asset-insurance.near",
        name: "Asset Insurance Oracle",
        description: "Real-time insurance premium calculations and risk assessment",
        successes: 120000,
        failures: 156,
        fee: {
            amount: "3500000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "asset_id:vehicle_123",
    },
    {
        id: "defi-rates.near",
        name: "DeFi Interest Rates",
        description: "Aggregated interest rates from major DeFi protocols",
        successes: 450000,
        failures: 67,
        fee: {
            amount: "1200000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "protocol:aave",
    },
    {
        id: "real-estate.near",
        name: "Real Estate Valuation",
        description: "Property valuation data from multiple real estate sources",
        successes: 75000,
        failures: 234,
        fee: {
            amount: "8000000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "property_id:ny_123",
    },
    {
        id: "air-quality.near",
        name: "Air Quality Index",
        description: "Real-time air quality measurements from global monitoring stations",
        successes: 280000,
        failures: 445,
        fee: {
            amount: "1800000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "city:shanghai",
    },
    {
        id: "commodity-prices.near",
        name: "Commodity Prices",
        description: "Real-time commodity price data from global markets",
        successes: 620000,
        failures: 89,
        fee: {
            amount: "2200000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "gold_spot",
    },
    {
        id: "domain-reputation.near",
        name: "Domain Reputation",
        description: "Website safety and reputation scoring system",
        successes: 890000,
        failures: 1200,
        fee: {
            amount: "1600000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "website.com",
    },
    {
        id: "traffic-data.near",
        name: "Traffic Data Feed",
        description: "Real-time traffic conditions and congestion data",
        successes: 420000,
        failures: 678,
        fee: {
            amount: "1900000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "highway_id:101",
    },
    {
        id: "energy-grid.near",
        name: "Energy Grid Monitor",
        description: "Power grid status and energy consumption metrics",
        successes: 180000,
        failures: 23,
        fee: {
            amount: "250000000000000000000000",
            token: "near"
        },
        exampleInput: "grid_sector:ny_manhattan",
    },
    {
        id: "health-insurance.near",
        name: "Health Insurance Claims",
        description: "Medical procedure pricing and insurance claim verification",
        successes: 95000,
        failures: 445,
        fee: {
            amount: "4500000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "procedure_code:12345",
    },
    {
        id: "agricultural-yield.near",
        name: "Agricultural Yield Data",
        description: "Crop yield and farming conditions monitoring",
        successes: 65000,
        failures: 78,
        fee: {
            amount: "2800000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "farm_id:iowa_123",
    },
    {
        id: "exchange-rates.near",
        name: "Exchange Rate Oracle",
        description: "Real-time currency exchange rates from major forex markets",
        successes: 780000,
        failures: 234,
        fee: {
            amount: "1700000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "USD/EUR",
    },
    {
        id: "smart-city.near",
        name: "Smart City Sensors",
        description: "Urban infrastructure monitoring and smart city metrics",
        successes: 320000,
        failures: 567,
        fee: {
            amount: "2100000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "sensor_type:parking",
    },
    {
        id: "supply-chain.near",
        name: "Supply Chain Tracker",
        description: "Global supply chain verification and tracking",
        successes: 250000,
        failures: 345,
        fee: {
            amount: "3200000",
            token: "usdt.tether-token.near"
        },
        exampleInput: "shipment_id:abc123",
    }
]; 
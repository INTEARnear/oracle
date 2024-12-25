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
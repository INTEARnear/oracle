import { ChakraProvider, extendTheme, ColorModeScript } from '@chakra-ui/react';
import type { AppProps } from 'next/app';
import Head from 'next/head';
import { createContext, useContext, useEffect, useState } from 'react';

interface TokenPrice {
    decimal: number;
    symbol: string;
}

interface TokenPrices {
    [key: string]: TokenPrice;
}

export const TokenPriceContext = createContext<TokenPrices>({});

const config = {
    initialColorMode: 'dark',
    useSystemColorMode: false,
};

const theme = extendTheme({
    config,
    styles: {
        global: {
            body: {
                bg: '#1a1a1a',
                minHeight: '100vh',
                display: 'flex',
                flexDirection: 'column',
            },
        },
    },
    colors: {
        purple: {
            50: '#f5f3ff',
            100: '#ede9fe',
            200: '#ddd6fe',
            300: '#c4b5fd',
            400: '#a78bfa',
            500: '#8b5cf6',
            600: '#7c3aed',
            700: '#6d28d9',
            800: '#5b21b6',
            900: '#4c1d95',
        },
    },
});

function MyApp({ Component, pageProps }: AppProps) {
    const [tokenPrices, setTokenPrices] = useState<TokenPrices>({});

    useEffect(() => {
        fetch('https://prices.intear.tech/list-token-price')
            .then(response => response.json())
            .then(data => setTokenPrices({
                ...data,
                near: {
                    decimal: 24,
                    symbol: "NEAR"
                }
            }))
            .catch(error => console.error('Error fetching token prices:', error));
    }, []);

    return (
        <ChakraProvider theme={theme}>
            <ColorModeScript initialColorMode="dark" />
            <Head>
                <title>Intear Oracle</title>
                <meta name="description" content="Decentralized data marketplace connecting data providers with consumers through secure and reliable oracles" />
                <link rel="icon" href="/favicon.ico" />
            </Head>
            <TokenPriceContext.Provider value={tokenPrices}>
                <Component {...pageProps} />
            </TokenPriceContext.Provider>
        </ChakraProvider>
    );
}

export default MyApp; 
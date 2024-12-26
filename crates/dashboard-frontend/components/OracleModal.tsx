import {
    Modal,
    ModalOverlay,
    ModalContent,
    ModalHeader,
    ModalBody,
    ModalCloseButton,
    Progress,
    Stack,
    Badge,
    Text,
    useColorModeValue,
    Heading,
    Divider,
    Box,
    Link,
} from '@chakra-ui/react';
import { Oracle, ORACLE_CONTRACT_ID } from '../api/oracles';
import { CopyableCode } from './CopyableCode';
import { useContext } from 'react';
import { TokenPriceContext } from '../pages/_app';

interface OracleModalProps {
    oracle: Oracle | null;
    isOpen: boolean;
    onClose: () => void;
}

const DEFAULT_EXAMPLE_INPUT = "<your input to the oracle>";

export const OracleModal = ({ oracle, isOpen, onClose }: OracleModalProps) => {
    const statBg = useColorModeValue('gray.100', 'gray.700');
    const tokenPrices = useContext(TokenPriceContext);

    if (!oracle) return null;

    const formatFee = () => {
        const tokenInfo = tokenPrices[oracle.fee.token];
        if (!tokenInfo) {
            return "";
        }
        const amount = Number(oracle.fee.amount) / Math.pow(10, tokenInfo.decimal);
        return `${amount.toFixed(2)} ${tokenInfo.symbol}`;
    };

    return (
        <Modal isOpen={isOpen} onClose={onClose} size="xl">
            <ModalOverlay backdropFilter="blur(10px)" />
            <ModalContent bg="gray.900">
                <ModalHeader>{oracle.name}</ModalHeader>
                <ModalCloseButton />
                <ModalBody pb={6}>
                    <Text mb={4}>{oracle.description}</Text>
                    <Progress
                        value={oracle.successes}
                        min={0}
                        max={oracle.successes + oracle.failures}
                        colorScheme="green"
                        mb={4}
                        borderRadius="full"
                        bg="red.400"
                    />
                    <Stack spacing={4} mb={6}>
                        <Stack direction="row" spacing={2} flexWrap="wrap">
                            <Badge bg={statBg} px={3} py={1}>
                                Used: {oracle.successes + oracle.failures} times
                            </Badge>
                            <Badge bg={statBg} px={3} py={1}>
                                Success Rate: {(oracle.successes / Math.max(1, oracle.successes + oracle.failures) * 100).toFixed(2)}%
                            </Badge>
                            <Badge bg={statBg} px={3} py={1}>
                                Fee: Up to {formatFee()}
                            </Badge>
                        </Stack>
                    </Stack>

                    <Divider my={6} />

                    <Stack spacing={6}>
                        <Box>
                            <Heading size="sm" mb={4}>CLI Usage Example</Heading>
                            <CopyableCode code={`near contract call-function as-transaction ${ORACLE_CONTRACT_ID} request json-args '{"producer_id":"${oracle.id}","request_data":"${(oracle.exampleInput ?? DEFAULT_EXAMPLE_INPUT).replace(/"/g, '\\"')}"}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as <YOUR_ACCOUNT_ID> network-config mainnet sign-with-keychain send`} language="shell" />
                            <Text>Note: The oracle may refund a portion of the fee if it wants to (some are usage-based, not per-request flat fee). The fee is fully refunded if the oracle fails to respond.</Text>
                        </Box>

                        <Box>
                            <Heading size="sm" mb={4}>Rust Integration Example</Heading>
                            <Text mb={4}>First, set up your Cargo.toml:</Text>
                            <CopyableCode
                                code={`[package]
name = "example"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
near-sdk = "5.5"
intear-oracle = { git = "https://github.com/INTEARnear/oracle", default-features = false }`}
                                language="toml"
                            />
                            <Text mt={4} mb={4}>Then implement the oracle consumer (src/lib.rs):</Text>
                            <CopyableCode
                                code={`use near_sdk::{env, near_bindgen, AccountId, Gas, Promise};
use intear_oracle::consumer::ext_oracle_consumer;

#[near_bindgen]
impl Contract {
    pub fn check_if_works(&self) -> Promise {
        ext_oracle_consumer::ext("${ORACLE_CONTRACT_ID}".parse().unwrap())
            .with_static_gas(Gas::from_tgas(10))
            // Attach a NEAR fee here (if it's in NEAR). Alternatively, you can use a subscription
            // model, deposit some NEAR or other fungible tokens, and just remove this comment.
            .request(
                "${oracle.id}".parse().unwrap(),
                "${(oracle.exampleInput ?? DEFAULT_EXAMPLE_INPUT).replace(/"/g, '\\"')}".to_string(),
            )
            .then(Self::ext(env::current_account_id()).on_response())
    }

    #[private] // the callback function must be private so that no one can just call it normally
    pub fn on_response(&self, #[callback_unwrap] response: Option<Response>) {
        let result = response.expect("Oracle didn't respond in time");
        let response: String = result.response_data;
        // Now you can do something with the response
    }
}`}
                                language="rust"
                            />
                        </Box>

                        <Box>
                            <Heading size="sm" mb={4}>Full Example</Heading>
                            <Text>
                                Check out the <Link href="https://github.com/INTEARnear/oracle/tree/7ddd6b9722e47481c463127d31282955e96ccbc3/crates/example-consumer" color="blue.400" isExternal>
                                    full example on GitHub
                                </Link>
                            </Text>
                        </Box>
                    </Stack>
                </ModalBody>
            </ModalContent>
        </Modal>
    );
}; 
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
} from '@chakra-ui/react';
import { Oracle } from '../data/mockOracles';
import { CopyableCode } from './CopyableCode';
import { useContext } from 'react';
import { TokenPriceContext } from '../pages/_app';

interface OracleModalProps {
    oracle: Oracle | null;
    isOpen: boolean;
    onClose: () => void;
}

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

    const cliExample = `# Query the oracle
oracle-cli query ${oracle.name.toLowerCase().replace(/ /g, '-')} \\
  --params '{"pair": "BTC/USD"}' \\
  --fee ${oracle.fee.amount} \\
  --token ${oracle.fee.token}

# Subscribe to updates
oracle-cli subscribe ${oracle.name.toLowerCase().replace(/ /g, '-')} \\
  --interval "1m" \\
  --callback "http://your-api.com/webhook"`;

    const rustExample = `use oracle_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OracleClient::new()
        .with_endpoint("${oracle.name.toLowerCase().replace(/ /g, '-')}")
        .with_token("${oracle.fee.token}")
        .with_fee(${oracle.fee.amount});

    // Query the oracle
    let response = client.query(json!({
        "pair": "BTC/USD"
    })).await?;

    println!("Price: {}", response.get("price").unwrap());
    Ok(())
}`;

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
                                Used times: {oracle.successes + oracle.failures}
                            </Badge>
                            <Badge bg={statBg} px={3} py={1}>
                                Success Rate: {(oracle.successes / Math.max(1, oracle.successes + oracle.failures) * 100).toFixed(2)}%
                            </Badge>
                            <Badge bg={statBg} px={3} py={1}>
                                Failure Rate: {(oracle.failures / Math.max(1, oracle.successes + oracle.failures) * 100).toFixed(2)}%
                            </Badge>
                            <Badge bg={statBg} px={3} py={1}>
                                Fee: {formatFee()}
                            </Badge>
                        </Stack>
                    </Stack>

                    <Divider my={6} />

                    <Stack spacing={6}>
                        <Box>
                            <Heading size="sm" mb={4}>CLI Usage Example</Heading>
                            <CopyableCode code={cliExample} language="shell" />
                        </Box>

                        <Box>
                            <Heading size="sm" mb={4}>Rust Integration Example</Heading>
                            <CopyableCode code={rustExample} language="rust" />
                        </Box>
                    </Stack>
                </ModalBody>
            </ModalContent>
        </Modal>
    );
}; 
import {
    Modal,
    ModalOverlay,
    ModalContent,
    ModalHeader,
    ModalBody,
    ModalCloseButton,
    Box,
    Text,
    VStack,
    Heading,
    Link,
} from '@chakra-ui/react';
import { CopyableCode } from './CopyableCode';

interface CreateOracleModalProps {
    isOpen: boolean;
    onClose: () => void;
}

export const CreateOracleModal = ({ isOpen, onClose }: CreateOracleModalProps) => {
    return (
        <Modal isOpen={isOpen} onClose={onClose} size="xl">
            <ModalOverlay backdropFilter="blur(10px)" />
            <ModalContent bg="gray.900">
                <ModalHeader>Create Your Own Oracle</ModalHeader>
                <ModalCloseButton />
                <ModalBody pb={6}>
                    <Text mb={4}>Follow these steps to register as a data provider in the Intear oracle marketplace:</Text>

                    <VStack spacing={6} align="stretch">
                        <Box>
                            <Heading size="sm" mb={2}>1. Install the NEAR CLI</Heading>
                            <CopyableCode
                                code="cargo install near-cli-rs"
                                language="bash"
                            />
                        </Box>

                        <Box>
                            <Heading size="sm" mb={2}>2. Initialize Your Oracle</Heading>
                            <CopyableCode
                                code={`near contract call-function as-transaction dev-unaudited-v0.oracle.intear.near add_producer json-args '{}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as <YOUR_ACCOUNT_ID> network-config mainnet sign-with-keychain send`}
                                language="bash"
                            />
                            <Text mt={2} fontSize="sm" color="gray.400">This will register your account on chain as a data provider.</Text>
                        </Box>

                        <Box>
                            <Heading size="sm" mb={2}>3. Configure Your Oracle Details</Heading>
                            <CopyableCode
                                code={`near contract call-function as-transaction dev-unaudited-v0.oracle.intear.near edit_producer_details json-args '{
  "name": "Weather Data",
  "description": "Real-time weather temperature data from multiple meteorological stations",
  "example_input": "Ukraine, Lviv"
}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as <YOUR_ACCOUNT_ID> network-config mainnet sign-with-keychain send`}
                                language="bash"
                            />
                        </Box>

                        <Box>
                            <Heading size="sm" mb={2}>4. Set a Fee</Heading>
                            <CopyableCode
                                code={`near contract call-function as-transaction dev-unaudited-v0.oracle.intear.near edit_producer_details json-args '{
  "fee": {
    "Near": {
      "prepaid_amount": "100000000000000000000000"
    }
  }
}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as <YOUR_ACCOUNT_ID> network-config mainnet sign-with-keychain send`}
                                language="bash"
                            />
                            Or use another fungible token:
                            <CopyableCode
                                code={`near contract call-function as-transaction dev-unaudited-v0.oracle.intear.near edit_producer_details json-args '{
  "fee": {
    "FungibleToken": {
      "token": "usdt.tether-token.near",
      "prepaid_amount": "1000000"
    }
  }
}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as <YOUR_ACCOUNT_ID> network-config mainnet sign-with-keychain send`}
                                language="bash"
                            />
                        </Box>

                        <Box>
                            <Heading size="sm" mb={2}>5. Refresh the Dashboard to see the changes</Heading>
                        </Box>

                        <Box>
                            <Text>Your oracle will appear in the marketplace within 5-10 seconds.</Text>
                            <Text mt={2}>
                                For more detailed documentation and support, visit{' '}
                                <Link href="https://docs.intear.tech/oracle" color="purple.500">
                                    our documentation
                                </Link>
                                .
                            </Text>
                        </Box>
                    </VStack>
                </ModalBody>
            </ModalContent>
        </Modal>
    );
}; 
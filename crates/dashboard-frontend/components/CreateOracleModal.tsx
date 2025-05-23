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
import { ORACLE_CONTRACT_ID } from '../api/oracles';

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
                                code={`near contract call-function as-transaction ${ORACLE_CONTRACT_ID} add_producer json-args '{}' prepaid-gas '100.0 Tgas' attached-deposit '0 NEAR' sign-as <YOUR_ACCOUNT_ID> network-config mainnet sign-with-keychain send`}
                                language="bash"
                            />
                            <Text mt={2} fontSize="sm" color="gray.400">This will register your account on chain as a data provider.</Text>
                        </Box>

                        <Box>
                            <Heading size="sm" mb={2}>3. Configure Your Oracle Details</Heading>
                            <CopyableCode
                                code={`near contract call-function as-transaction ${ORACLE_CONTRACT_ID} edit_producer_details json-args '{
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
                                code={`near contract call-function as-transaction ${ORACLE_CONTRACT_ID} edit_producer_details json-args '{
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
                                code={`near contract call-function as-transaction ${ORACLE_CONTRACT_ID} edit_producer_details json-args '{
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
                            <Heading size="sm" mb={2}>5. Go back to this page to see the changes</Heading>
                        </Box>

                        <Box>
                            <Text>Your oracle will appear in the marketplace within 5-10 seconds.</Text>
                            <Text mt={2}>
                                For more detailed documentation and support, visit{' '}
                                <Link href="https://docs.intear.tech/docs/oracle" color="blue.400" isExternal>
                                    our documentation
                                </Link>
                                .
                            </Text>
                        </Box>

                        <Box>
                            <Heading size="sm" mb={2}>6. Run a node</Heading>
                            <Text>To make your node work, you need to run a node service. You can find an example of a node <Link href="https://github.com/INTEARnear/oracle/blob/0fed49b148c62173dc860ab75c398eb88253bdc1/crates/gpt4o-node/src/main.rs" color="blue.400" isExternal>here</Link>.</Text>
                        </Box>
                    </VStack>
                </ModalBody>
            </ModalContent>
        </Modal>
    );
}; 

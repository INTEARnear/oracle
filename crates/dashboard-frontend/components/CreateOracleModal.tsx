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
                    <Text mb={4}>Follow these steps to register as a data provider in the marketplace:</Text>

                    <VStack spacing={6} align="stretch">
                        <Box>
                            <Heading size="sm" mb={2}>1. Install the Oracle CLI</Heading>
                            <CopyableCode
                                code="npm install -g oracle-marketplace-cli"
                                language="shell"
                            />
                        </Box>

                        <Box>
                            <Heading size="sm" mb={2}>2. Initialize Your Oracle Configuration</Heading>
                            <CopyableCode
                                code="oracle-cli init"
                                language="shell"
                            />
                            <Text mt={2} fontSize="sm" color="gray.400">This will create a oracle-config.json file in your current directory.</Text>
                        </Box>

                        <Box>
                            <Heading size="sm" mb={2}>3. Configure Your Oracle Details</Heading>
                            <CopyableCode
                                code={`{
  "name": "Your Oracle Name",
  "description": "Description of your data service",
  "endpoint": "https://api.youroracle.com/data",
  "fee": {
    "amount": "0.1",
    "token": "LINK"
  },
  "updateFrequency": "1m",
  "responseFormat": "JSON"
}`}
                                language="json"
                            />
                        </Box>

                        <Box>
                            <Heading size="sm" mb={2}>4. Deploy Your Oracle</Heading>
                            <CopyableCode
                                code="oracle-cli deploy --network mainnet"
                                language="shell"
                            />
                        </Box>

                        <Box>
                            <Heading size="sm" mb={2}>5. Verify Your Oracle</Heading>
                            <CopyableCode
                                code="oracle-cli verify --oracle-id YOUR_ORACLE_ID"
                                language="shell"
                            />
                        </Box>

                        <Box>
                            <Text>Once verified, your oracle will appear in the marketplace within 5-10 minutes.</Text>
                            <Text mt={2}>
                                For more detailed documentation and support, visit{' '}
                                <Link href="https://docs.oraclemarketplace.com" color="purple.500">
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
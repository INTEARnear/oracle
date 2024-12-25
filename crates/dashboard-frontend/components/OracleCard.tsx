import { Box, Text, Progress, Stack, Badge, useColorModeValue } from '@chakra-ui/react';
import { Oracle } from '../data/mockOracles';

interface OracleCardProps {
    oracle: Oracle;
    onClick: () => void;
}

export const OracleCard = ({ oracle, onClick }: OracleCardProps) => {
    const cardBg = useColorModeValue('white', 'gray.800');
    const statBg = useColorModeValue('gray.100', 'gray.700');

    return (
        <Box
            bg={cardBg}
            p={6}
            borderRadius="lg"
            boxShadow="lg"
            cursor="pointer"
            transition="all 0.3s"
            _hover={{
                transform: 'translateY(-5px)',
                boxShadow: 'xl',
                borderColor: 'purple.500',
            }}
            onClick={onClick}
            border="1px solid"
            borderColor="gray.200"
        >
            <Text fontSize="xl" fontWeight="bold" mb={2}>
                {oracle.name}
            </Text>
            <Text color="gray.500" mb={4}>
                {oracle.description}
            </Text>
            <Progress
                value={oracle.successes / (oracle.successes + oracle.failures) * 100}
                colorScheme="green"
                mb={4}
                borderRadius="full"
                bg="red.400"
            />
            <Stack direction="row" spacing={2} flexWrap="wrap">
                <Badge bg={statBg} px={3} py={1}>
                    👥 {oracle.successes + oracle.failures} times used
                </Badge>
                <Badge bg={statBg} px={3} py={1}>
                    ✅ {oracle.successes / Math.max(1, oracle.successes + oracle.failures) * 100}% uptime
                </Badge>
                <Badge bg={statBg} px={3} py={1}>
                    💰 {oracle.fee.amount} {oracle.fee.token}
                </Badge>
            </Stack>
        </Box>
    );
}; 
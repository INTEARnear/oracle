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
        value={oracle.successRate}
        colorScheme="green"
        mb={4}
        borderRadius="full"
      />
      <Stack direction="row" spacing={2} flexWrap="wrap">
        <Badge bg={statBg} px={3} py={1}>
          👥 {oracle.users.toLocaleString()} users
        </Badge>
        <Badge bg={statBg} px={3} py={1}>
          ✅ {oracle.successRate}%
        </Badge>
        <Badge bg={statBg} px={3} py={1}>
          💰 {oracle.fee.amount} {oracle.fee.token}
        </Badge>
      </Stack>
    </Box>
  );
}; 
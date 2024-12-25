import { useState, useEffect } from 'react';
import {
  Box,
  Container,
  Input,
  Button,
  SimpleGrid,
  Heading,
  Text,
  useDisclosure,
  InputGroup,
  InputLeftElement,
  Stack,
  Flex,
  HStack,
} from '@chakra-ui/react';
import { SearchIcon, ChevronLeftIcon, ChevronRightIcon } from '@chakra-ui/icons';
import { OracleCard } from '../components/OracleCard';
import { OracleModal } from '../components/OracleModal';
import { CreateOracleModal } from '../components/CreateOracleModal';
import { Oracle, fetchOracles } from '../api/oracles';

const ITEMS_PER_PAGE = 12;

export default function Home() {
  const [searchTerm, setSearchTerm] = useState('');
  const [currentPage, setCurrentPage] = useState(1);
  const [selectedOracle, setSelectedOracle] = useState<Oracle | null>(null);
  const [oracles, setOracles] = useState<Oracle[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const {
    isOpen: isOracleModalOpen,
    onOpen: onOracleModalOpen,
    onClose: onOracleModalClose,
  } = useDisclosure();
  const {
    isOpen: isCreateModalOpen,
    onOpen: onCreateModalOpen,
    onClose: onCreateModalClose,
  } = useDisclosure();

  useEffect(() => {
    const loadOracles = async () => {
      setIsLoading(true);
      const data = await fetchOracles();
      setOracles(data);
      setIsLoading(false);
    };

    loadOracles();
    const interval = setInterval(loadOracles, 10000); // Refresh every 10 seconds

    return () => clearInterval(interval);
  }, []);

  const filteredOracles = oracles.filter(
    oracle =>
      oracle.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      oracle.description.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const totalPages = Math.ceil(filteredOracles.length / ITEMS_PER_PAGE);
  const startIndex = (currentPage - 1) * ITEMS_PER_PAGE;
  const paginatedOracles = filteredOracles.slice(startIndex, startIndex + ITEMS_PER_PAGE);

  const handlePageChange = (newPage: number) => {
    setCurrentPage(newPage);
    window.scrollTo({ top: 0, behavior: 'smooth' });
  };

  const handleOracleClick = (oracle: Oracle) => {
    setSelectedOracle(oracle);
    onOracleModalOpen();
  };

  return (
    <Flex direction="column" minH="100vh">
      <Box as="header" py={8} borderBottom="1px solid" borderColor="gray.800">
        <Container maxW="container.xl">
          <Stack spacing={4} align="center">
            <Heading color="blue.400" size="2xl">Intear Oracle</Heading>
            <Text color="gray.500" fontSize="lg" textAlign="center">
              Decentralized data marketplace connecting data providers with consumers through an oracle contract
            </Text>
          </Stack>
        </Container>
      </Box>

      <Box flex="1">
        <Container maxW="container.xl">
          <Stack spacing={8} py={8}>
            <Flex gap={4}>
              <InputGroup flex={1}>
                <InputLeftElement pointerEvents="none">
                  <SearchIcon color="gray.500" boxSize={5} mt={2} ml={2} />
                </InputLeftElement>
                <Input
                  placeholder="Search oracles ..."
                  value={searchTerm}
                  onChange={e => setSearchTerm(e.target.value)}
                  size="lg"
                  bg="gray.800"
                />
              </InputGroup>
              <Button colorScheme="green" size="lg" onClick={onCreateModalOpen}>
                Create an Oracle
              </Button>
            </Flex>

            <SimpleGrid columns={{ base: 1, md: 2, lg: 3 }} spacing={6}>
              {paginatedOracles.map(oracle => (
                <OracleCard
                  key={oracle.id}
                  oracle={oracle}
                  onClick={() => handleOracleClick(oracle)}
                />
              ))}
            </SimpleGrid>

            {totalPages > 1 && (
              <HStack justify="center" spacing={2}>
                <Button
                  leftIcon={<ChevronLeftIcon />}
                  onClick={() => handlePageChange(currentPage - 1)}
                  isDisabled={currentPage === 1}
                  variant="outline"
                >
                  Previous
                </Button>
                
                {Array.from({ length: totalPages }, (_, i) => i + 1).map((page) => (
                  <Button
                    key={page}
                    onClick={() => handlePageChange(page)}
                    variant={currentPage === page ? "solid" : "outline"}
                    colorScheme={currentPage === page ? "purple" : "gray"}
                  >
                    {page}
                  </Button>
                ))}

                <Button
                  rightIcon={<ChevronRightIcon />}
                  onClick={() => handlePageChange(currentPage + 1)}
                  isDisabled={currentPage === totalPages}
                  variant="outline"
                >
                  Next
                </Button>
              </HStack>
            )}

            {filteredOracles.length === 0 && (
              <Text textAlign="center" color="gray.500">
                No oracles found matching your search criteria
              </Text>
            )}
          </Stack>
        </Container>
      </Box>

      <Box as="footer" py={8} borderTop="1px solid" borderColor="gray.800" mt="auto">
        <Container maxW="container.xl">
          <Stack direction="row" spacing={6} justify="center">
            <Button variant="link" as="a" href="https://docs.intear.tech/oracle">
              Documentation
            </Button>
            <Button variant="link" as="a" href="https://intear.tech">
              Website
            </Button>
            <Button variant="link" as="a" href="https://github.com/INTEARnear/oracle/">
              GitHub
            </Button>
            <Button variant="link" as="a" href="https://t.me/intearchat">
              Telegram
            </Button>
          </Stack>
        </Container>
      </Box>

      <OracleModal
        oracle={selectedOracle}
        isOpen={isOracleModalOpen}
        onClose={onOracleModalClose}
      />

      <CreateOracleModal
        isOpen={isCreateModalOpen}
        onClose={onCreateModalClose}
      />
    </Flex>
  );
} 
import { Box, IconButton, useClipboard, Text } from '@chakra-ui/react';
import { CopyIcon, CheckIcon } from '@chakra-ui/icons';
import { Highlight, themes } from 'prism-react-renderer';

// Map our simplified language names to Prism's language names
const languageMap: { [key: string]: string } = {
  shell: 'bash',
  rust: 'rust',
  json: 'json',
  text: 'text',
};

interface CopyableCodeProps {
  code: string;
  language?: string;
}

export const CopyableCode = ({ code, language = 'text' }: CopyableCodeProps) => {
  const { hasCopied, onCopy } = useClipboard(code);
  const prismLanguage = languageMap[language] || 'text';

  return (
    <Box position="relative" mb={4}>
      <IconButton
        aria-label="Copy code"
        icon={hasCopied ? <CheckIcon /> : <CopyIcon />}
        position="absolute"
        top={2}
        right={2}
        size="sm"
        onClick={onCopy}
        colorScheme={hasCopied ? "green" : "gray"}
        zIndex={1}
      />
      {language && (
        <Text
          position="absolute"
          top={2}
          left={3}
          fontSize="xs"
          color="gray.500"
          textTransform="uppercase"
          zIndex={1}
        >
          {language}
        </Text>
      )}
      <Highlight
        theme={themes.nightOwl}
        code={code.trim()}
        language={prismLanguage}
      >
        {({ className, style, tokens, getLineProps, getTokenProps }) => (
          <Box
            as="pre"
            className={className}
            style={style}
            p={4}
            pt={language ? 10 : 4}
            borderRadius="md"
            bg="gray.800"
            fontSize="sm"
            overflow="auto"
          >
            {tokens.map((line, i) => (
              <div key={i} {...getLineProps({ line, key: i })}>
                {line.map((token, key) => (
                  <span key={key} {...getTokenProps({ token, key })} />
                ))}
              </div>
            ))}
          </Box>
        )}
      </Highlight>
    </Box>
  );
}; 
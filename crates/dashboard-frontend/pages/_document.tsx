import { Html, Head, Main, NextScript } from 'next/document';

export default function Document() {
  return (
    <Html lang="en">
      <Head />
      <body>
        <script
          dangerouslySetInnerHTML={{
            __html: `
              (function() {
                const colorMode = localStorage.getItem('chakra-ui-color-mode');
                if (colorMode === 'dark' || (!colorMode && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
                  document.documentElement.style.setProperty('background-color', '#1a1a1a');
                  document.body.style.setProperty('background-color', '#1a1a1a');
                }
              })();
            `,
          }}
        />
        <Main />
        <NextScript />
      </body>
    </Html>
  );
} 
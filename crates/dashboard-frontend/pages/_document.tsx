import { Html, Head, Main, NextScript } from 'next/document';

export default function Document() {
  return (
    <Html lang="en">
      <Head>
        <link
          href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap"
          rel="stylesheet"
        />
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
      </Head>
      <body>
        <Main />
        <NextScript />
      </body>
    </Html>
  );
} 
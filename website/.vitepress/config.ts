import { defineConfig } from 'vitepress';

// https://vitepress.dev/reference/site-config
export default defineConfig({
  title: 'Escrin',
  description:
    'Escrin allows you to extend smart contracts with flexible, off-chain, private computation.',
  head: [
    ['link', { rel: 'shortcut icon', href: '/logo.svg' }],
    ['meta', { name: 'theme-color', content: '#eeaa00' }],
  ],
  sitemap: {
    hostname: 'https://escrin.org',
  },
  themeConfig: {
    logo: '/logo.svg',
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Docs', link: '/docs/' },
      { text: 'Community', link: 'https://escrin.org/discord' },
    ],

    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'What is Escrin?', link: '/docs/' },
          // { text: 'Technology', link: '/docs/guide/technology' },
          // { text: 'Getting Started', link: '/docs/guide/getting-started' },
        ],
      },
      {
        text: 'Tutorial',
        items: [
          { text: '1. Create On-Chain Tasks', link: '/docs/tutorial/first-task' },
          { text: '2. Fulfill Tasks Using Workers', link: '/docs/tutorial/first-worker' },
          { text: '3. Create & Acquire an Identity', link: '/docs/tutorial/first-identity' },
          { text: '4. Secrets & Trusted Workers', link: '/docs/tutorial/secret-worker' },
        ],
      },
      {
        text: 'Network',
        items: [
          { text: 'Whitepaper', link: '/docs/network/' },
          // { text: 'Escrin Observer', link: 'https://observer.escrin.org' },
        ],
      },
      {
        text: 'Applications',
        items: [
          { text: 'Nanobridges', link: '/docs/apps/nanobridges' },
          // { text: 'Games', link: '/docs/apps/games' },
          // { text: 'AI', link: '/docs/apps/ai' },
        ],
      },
      {
        text: 'Reference',
        items: [
          { text: 'Smart Workers', link: '/docs/reference/worker' },
          // { text: 'Solidity Library', link: '/docs/reference/solidity' },
          { text: 'Escrin Runner', link: '/docs/reference/runner' },
          { text: 'Simple Secret Sharing Server (SSSS)', link: '/docs/reference/ssss' },
        ],
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/escrin/escrin' },
      { icon: 'discord', link: 'https://escrin.org/discord' },
      {
        icon: {
          svg: '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 50 50"><path d="M46.137 6.552c-.75-.636-1.928-.727-3.146-.238h-.002c-1.281.514-36.261 15.518-37.685 16.131-.259.09-2.521.934-2.288 2.814.208 1.695 2.026 2.397 2.248 2.478l8.893 3.045c.59 1.964 2.765 9.21 3.246 10.758.3.965.789 2.233 1.646 2.494.752.29 1.5.025 1.984-.355l5.437-5.043 8.777 6.845.209.125c.596.264 1.167.396 1.712.396.421 0 .825-.079 1.211-.237 1.315-.54 1.841-1.793 1.896-1.935l6.556-34.077c.4-1.82-.156-2.746-.694-3.201M22 32l-3 8-3-10 23-17z"/></svg>',
        },
        link: 'https://escrin.org/telegram',
      },
      { icon: 'x', link: 'https://escrin.org/twitter' },
    ],

    algolia: {
      indexName: 'escrin',
      appId: 'ZNRK6V99NY',
      apiKey: 'bf4aaa6ca6d33a85474cb33796497f0f',
    },
  },
});

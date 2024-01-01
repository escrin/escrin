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
    hostname: 'https://escrin.org'
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
        ],
      },
      {
        text: 'Tutorial',
        items: [
          { text: '1. Create On-Chain Tasks', link: '/docs/tutorial/first-task' },
          { text: '2. Fulfill Tasks Using a Worker', link: '/docs/tutorial/first-worker' },
          { text: '3. Create and Acquire an Identity', link: '/docs/tutorial/first-identity' },
          { text: '4. Process Secrets Using a Worker', link: '/docs/tutorial/secret-worker' },
        ],
      },
      {
        text: 'Reference',
        items: [
          { text: 'Smart Workers', link: '/docs/reference/worker' },
          // { text: 'Solidity Library', link: '/docs/reference/solidity' },
        ],
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/escrin/escrin' },
      { icon: 'discord', link: 'https://escrin.org/discord' },
      { icon: 'twitter', link: 'https://twitter.com/EnshrineCC' }
    ],

    algolia: {
      indexName: 'escrin',
      appId: 'ZNRK6V99NY',
      apiKey: 'bf4aaa6ca6d33a85474cb33796497f0f',
    },
  },
});

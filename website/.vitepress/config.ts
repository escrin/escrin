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
  themeConfig: {
    logo: '/logo.svg',
    // https://vitepress.dev/reference/default-theme-config
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Docs', link: '/docs/' },
      { text: 'Community', link: 'https://enshrine.ai/discord' },
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
          { text: '2. Fulfill Tasks using a Worker', link: '/docs/tutorial/first-worker' },
        ],
      },
      {
        text: 'Reference',
        items: [
          { text: 'Smart Workers', link: '/docs/reference/worker' },
          { text: 'Solidity Library', link: '/docs/reference/solidity' },
        ],
      },
    ],

    socialLinks: [
      { icon: 'github', link: 'https://github.com/escrin/escrin' },
      { icon: 'discord', link: 'https://enshrine.ai' },
      { icon: 'twitter', link: 'https://twitter.com/EnshrineCC' }
    ],

    algolia: {
      indexName: 'escrin',
      appId: 'ZNRK6V99NY',
      apiKey: 'ab132c7d3e214170645c7e45a41094dd',
    },
  },
});

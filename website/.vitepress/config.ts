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
          { text: 'My First Task', link: '/docs/first-task' },
        ],
      },
      {
        text: 'Worker API',
        items: [
          { text: 'Overview', link: '/docs/api/' },
          { text: 'Key Management', link: '/docs/api/key-management' },
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

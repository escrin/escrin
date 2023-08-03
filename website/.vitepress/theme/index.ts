// https://vitepress.dev/guide/custom-theme
import { h } from 'vue';
import Theme from 'vitepress/theme';

import './style.css';
import SponsorsBox from './SponsorsBox.vue';

export default {
  extends: Theme,
  Layout: () => {
    return h(Theme.Layout, null, {
      // https://vitepress.dev/guide/extending-default-theme#layout-slots
      'home-features-after': () => h(SponsorsBox),
    });
  },
  enhanceApp({ app, router, siteData }) {},
};

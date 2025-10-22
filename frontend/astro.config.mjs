import { defineConfig } from 'astro/config';
import svelte from '@astrojs/svelte';
import tailwind from '@astrojs/tailwind';
import node from '@astrojs/node';

export default defineConfig({
  output: 'hybrid',
  adapter: node({
    mode: 'standalone'
  }),
  integrations: [svelte(), tailwind()],
  server: {
    port: 3000,
    host: true
  }
});

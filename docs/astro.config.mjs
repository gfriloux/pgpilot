import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://gfriloux.github.io/',
  base: '/pgpilot/',
  integrations: [
    starlight({
      title: 'PGPilot',
      description: 'PGP key manager — desktop app for Linux',
      defaultLocale: 'root',
      locales: {
        root: {
          label: 'English',
          lang: 'en',
        },
        fr: {
          label: 'Français',
          lang: 'fr',
        },
      },
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/gfriloux/pgpilot' },
      ],
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Introduction', link: '/' },
            { label: 'Installation', link: '/installation/' },
            { label: 'Quickstart', link: '/quickstart/' },
          ],
        },
        {
          label: 'Features',
          items: [
            { label: 'Key Management', link: '/key-management/' },
            { label: 'Keyserver & Sharing', link: '/keyserver/' },
            { label: 'File Operations', link: '/file-operations/' },
            { label: 'YubiKey & Smartcard', link: '/smartcard/' },
            { label: 'Chat', link: '/chat/' },
            { label: 'Settings', link: '/settings/' },
          ],
        },
        {
          label: 'Reference',
          items: [
            { label: 'Security', link: '/security/' },
            { label: 'FAQ', link: '/faq/' },
            { label: 'Troubleshooting', link: '/troubleshooting/' },
          ],
        },
      ],
    }),
  ],
});

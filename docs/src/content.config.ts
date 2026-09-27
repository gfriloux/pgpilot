import { defineCollection } from 'astro:content';
import { docsLoader, i18nLoader } from '@astrojs/starlight/loaders';
import { docsSchema, i18nSchema } from '@astrojs/starlight/schema';

export const collections = {
  docs: defineCollection({ loader: docsLoader(), schema: docsSchema() }),
  // Starlight 0.42 looks up an `i18n` collection as soon as `locales` is set in
  // astro.config.mjs, and warns at build time when it is missing. No UI string is
  // overridden yet — the dictionaries are empty placeholders and the entry point
  // for future translations of Starlight's own labels.
  i18n: defineCollection({ loader: i18nLoader(), schema: i18nSchema() }),
};

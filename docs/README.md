# PGPilot Documentation

This is the Astro + Starlight documentation site for [pgpilot](https://github.com/gfriloux/pgpilot), a graphical PGP key manager for Linux.

## Directory Structure

```
docs/
├── astro.config.mjs       — Astro + Starlight configuration
├── package.json           — Dependencies and build scripts
├── tsconfig.json          — TypeScript configuration
├── src/
│   ├── content/
│   │   └── docs/
│   │       ├── en/        — English documentation
│   │       └── fr/        — French documentation
│   └── env.d.ts
└── public/
    └── screenshots/       — Application screenshots
```

## Installation

```bash
cd docs
npm install
```

## Development

```bash
npm run dev
```

The documentation will be available at `http://localhost:3000` (or similar).

## Building

```bash
npm run build
```

The static site is generated in `docs/dist/`.

## Deployment

Documentation is automatically deployed to GitHub Pages when changes are pushed to the `main` branch. The workflow is defined in `.github/workflows/docs.yml`.

## Content

Documentation is written in Markdown/MDX and organized by language:

- **English** (`src/content/docs/en/`) — default language
- **French** (`src/content/docs/fr/`) — French translation

### Adding a new page

1. Create a new `.mdx` file in `src/content/docs/en/` (or `fr/` for French)
2. Add frontmatter:
   ```mdx
   ---
   title: Your Page Title
   description: Short description for SEO
   ---
   ```
3. Update `astro.config.mjs` sidebar if it's a new section

### Using images

Images should be placed in `public/screenshots/`. Reference them in content as:

```mdx
![My screenshot](/pgpilot/screenshots/my-screenshot.png)
```

Note the `/pgpilot/` base path (configured in `astro.config.mjs` as `base: '/pgpilot/'`).

## Localization

The documentation supports English (en) and French (fr) via Starlight's built-in i18n support.

- English content: `src/content/docs/en/`
- French content: `src/content/docs/fr/`

Both languages share the same file structure and frontmatter.

## Contributing

To contribute to the documentation:

1. Fork the repository
2. Create a branch: `git checkout -b improve-docs`
3. Edit Markdown/MDX files in `src/content/docs/`
4. Test locally: `npm run dev`
5. Commit and push your changes
6. Open a Pull Request

## More Information

- [Astro documentation](https://docs.astro.build)
- [Starlight documentation](https://starlight.astro.build)
- [pgpilot repository](https://github.com/gfriloux/pgpilot)

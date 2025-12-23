# Guts Documentation Site

A comprehensive documentation site for the Guts decentralized code collaboration platform, built with [VitePress](https://vitepress.dev).

## Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Preview production build
npm run preview
```

## Structure

```
docs-site/
├── .vitepress/
│   ├── config.ts          # VitePress configuration
│   └── theme/
│       ├── index.ts       # Custom theme
│       └── custom.css     # Brand styling
├── public/
│   └── logo.svg           # Site logo
├── index.md               # Landing page
├── developer/             # Developer documentation
│   ├── api/               # API reference
│   ├── guides/            # Tutorials and guides
│   ├── quickstart/        # Getting started
│   └── sdks/              # SDK documentation
├── operator/              # Operator documentation
│   ├── configuration/     # Configuration reference
│   ├── installation/      # Docker, K8s, bare metal
│   ├── operations/        # Monitoring, backup, upgrades
│   └── runbooks/          # Operational runbooks
└── architecture/          # Architecture documentation
    ├── adr/               # Architecture Decision Records
    ├── prd.md             # Product Requirements
    └── roadmap.md         # Project roadmap
```

## Deployment

The documentation is automatically deployed to GitHub Pages on push to `main`.

**Live URL**: https://abdelstark.github.io/guts/docs/

## Design

The documentation follows the **Obsidian Minimalism** design system:

- **Colors**: Ember (#FF3B2E) primary, Cipher (#6AE4FF) accent
- **Theme**: Dark mode optimized with Ink (#07080B) background
- **Typography**: Clean, readable, code-focused

## Contributing

1. Edit markdown files in the appropriate directory
2. Run `npm run dev` to preview changes
3. Submit a PR to the main repository

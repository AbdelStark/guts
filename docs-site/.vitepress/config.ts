import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Guts',
  description: 'Decentralized Code Collaboration Platform',

  head: [
    ['link', { rel: 'icon', type: 'image/svg+xml', href: '/logo.svg' }],
    ['meta', { name: 'theme-color', content: '#FF3B2E' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Guts Documentation' }],
    ['meta', { property: 'og:description', content: 'Decentralized, censorship-resistant code collaboration platform' }],
    ['meta', { property: 'og:url', content: 'https://abdelstark.github.io/guts/docs/' }],
  ],

  base: '/guts/docs/',

  cleanUrls: true,
  lastUpdated: true,
  ignoreDeadLinks: true,

  themeConfig: {
    logo: '/logo.svg',

    nav: [
      { text: 'Home', link: '/' },
      { text: 'Developer', link: '/developer/', activeMatch: '/developer/' },
      { text: 'Operator', link: '/operator/', activeMatch: '/operator/' },
      { text: 'Architecture', link: '/architecture/', activeMatch: '/architecture/' },
      {
        text: 'Resources',
        items: [
          { text: 'API Reference', link: '/developer/api/' },
          { text: 'SDKs', link: '/developer/sdks/' },
          { text: 'GitHub', link: 'https://github.com/AbdelStark/guts' },
          { text: 'Web App', link: 'https://abdelstark.github.io/guts/' },
        ]
      }
    ],

    sidebar: {
      '/developer/': [
        {
          text: 'Getting Started',
          collapsed: false,
          items: [
            { text: 'Introduction', link: '/developer/' },
            { text: 'Quickstart', link: '/developer/quickstart/' },
          ]
        },
        {
          text: 'API Reference',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/developer/api/' },
            { text: 'Authentication', link: '/developer/api/authentication' },
            { text: 'Repositories', link: '/developer/api/repositories' },
            { text: 'Pull Requests', link: '/developer/api/pull-requests' },
            { text: 'Issues', link: '/developer/api/issues' },
            { text: 'Organizations', link: '/developer/api/organizations' },
            { text: 'Consensus', link: '/developer/api/consensus' },
            { text: 'Releases', link: '/developer/api/releases' },
          ]
        },
        {
          text: 'SDKs',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/developer/sdks/' },
            { text: 'TypeScript', link: '/developer/sdks/typescript' },
            { text: 'Python', link: '/developer/sdks/python' },
          ]
        },
        {
          text: 'Guides',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/developer/guides/' },
            { text: 'Migration from GitHub', link: '/developer/guides/migration' },
            { text: 'Webhooks', link: '/developer/guides/webhooks' },
            { text: 'CI/CD Integration', link: '/developer/guides/ci-cd' },
          ]
        },
      ],

      '/operator/': [
        {
          text: 'Getting Started',
          collapsed: false,
          items: [
            { text: 'Introduction', link: '/operator/' },
            { text: 'Quickstart', link: '/operator/quickstart' },
            { text: 'Requirements', link: '/operator/requirements' },
            { text: 'Architecture', link: '/operator/architecture' },
          ]
        },
        {
          text: 'Installation',
          collapsed: false,
          items: [
            { text: 'Docker', link: '/operator/installation/docker' },
            { text: 'Kubernetes', link: '/operator/installation/kubernetes' },
            { text: 'Bare Metal', link: '/operator/installation/bare-metal' },
            { text: 'Systemd', link: '/operator/installation/systemd' },
          ]
        },
        {
          text: 'Configuration',
          collapsed: false,
          items: [
            { text: 'Reference', link: '/operator/configuration/reference' },
          ]
        },
        {
          text: 'Operations',
          collapsed: false,
          items: [
            { text: 'Monitoring', link: '/operator/operations/monitoring' },
            { text: 'Backup & Recovery', link: '/operator/operations/backup' },
            { text: 'Upgrades', link: '/operator/operations/upgrades' },
          ]
        },
        {
          text: 'Runbooks',
          collapsed: true,
          items: [
            { text: 'Overview', link: '/operator/runbooks/' },
            { text: 'Node Not Syncing', link: '/operator/runbooks/node-not-syncing' },
            { text: 'Consensus Stuck', link: '/operator/runbooks/consensus-stuck' },
            { text: 'High Memory', link: '/operator/runbooks/high-memory' },
            { text: 'Disk Full', link: '/operator/runbooks/disk-full' },
            { text: 'Data Corruption', link: '/operator/runbooks/data-corruption' },
            { text: 'Key Rotation', link: '/operator/runbooks/key-rotation' },
            { text: 'Emergency Shutdown', link: '/operator/runbooks/emergency-shutdown' },
          ]
        },
      ],

      '/architecture/': [
        {
          text: 'Overview',
          collapsed: false,
          items: [
            { text: 'Introduction', link: '/architecture/' },
            { text: 'Product Requirements', link: '/architecture/prd' },
            { text: 'Roadmap', link: '/architecture/roadmap' },
          ]
        },
        {
          text: 'Architecture Decision Records',
          collapsed: false,
          items: [
            { text: 'Overview', link: '/architecture/adr/' },
            { text: 'ADR-001: Commonware Primitives', link: '/architecture/adr/001-commonware-primitives' },
            { text: 'ADR-002: Content-Addressed Storage', link: '/architecture/adr/002-content-addressed-storage' },
            { text: 'ADR-003: Git Protocol', link: '/architecture/adr/003-git-protocol-implementation' },
            { text: 'ADR-004: Collaboration Model', link: '/architecture/adr/004-collaboration-data-model' },
            { text: 'ADR-005: Permissions', link: '/architecture/adr/005-permission-hierarchy' },
            { text: 'ADR-006: API Design', link: '/architecture/adr/006-api-design' },
            { text: 'ADR-007: Crate Architecture', link: '/architecture/adr/007-crate-architecture' },
          ]
        },
      ],
    },

    socialLinks: [
      { icon: 'github', link: 'https://github.com/AbdelStark/guts' }
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright 2025 Guts Contributors'
    },

    editLink: {
      pattern: 'https://github.com/AbdelStark/guts/edit/main/docs-site/:path',
      text: 'Edit this page on GitHub'
    },

    search: {
      provider: 'local',
      options: {
        detailedView: true
      }
    },

    outline: {
      level: [2, 3],
      label: 'On this page'
    },

    docFooter: {
      prev: 'Previous',
      next: 'Next'
    },

    darkModeSwitchLabel: 'Theme',
    sidebarMenuLabel: 'Menu',
    returnToTopLabel: 'Return to top',
    langMenuLabel: 'Language',
  },

  markdown: {
    lineNumbers: true,
    theme: {
      light: 'github-light',
      dark: 'github-dark'
    }
  },

  vite: {
    css: {
      preprocessorOptions: {
        scss: {
          api: 'modern'
        }
      }
    }
  }
})

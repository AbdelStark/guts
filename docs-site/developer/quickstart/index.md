# Quickstart Guide

Get started with Guts in 5 minutes!

## Prerequisites

- Git installed
- A Guts account (or use a public node)

## Step 1: Install the CLI

::: code-group

```bash [Cargo]
cargo install guts-cli
```

```bash [Script]
curl -sSL https://get.guts.network | sh
```

:::

## Step 2: Create Your Identity

Every Guts user has an Ed25519 keypair for cryptographic identity:

```bash
# Generate a new Ed25519 keypair
guts identity generate

# This creates ~/.guts/identity.json
```

Your identity includes:
- **Public key** - Your unique identifier on the network
- **Private key** - Used to sign commits and operations

::: warning
Keep your private key secure. Never share it with anyone.
:::

## Step 3: Get a Token

Visit your Guts dashboard to create a personal access token, or use the CLI:

```bash
guts auth login
```

This will:
1. Open your browser to the authentication page
2. Generate a personal access token
3. Store it securely in `~/.guts/credentials`

## Step 4: Clone a Repository

```bash
git clone https://guts.network/owner/repo.git
```

Git operations work exactly like you're used to!

## Step 5: Create Your First Repository

```bash
# Create a new repository
guts repo create my-project

# Or initialize an existing directory
cd my-project
guts init
git remote add origin https://guts.network/you/my-project.git
git push -u origin main
```

## Verify Your Setup

```bash
# Check CLI is working
guts --version

# Check your identity
guts identity show

# List your repositories
guts repo list
```

## Using with Git

Guts is fully Git-compatible. Use your normal Git workflow:

```bash
# Clone
git clone https://guts.network/owner/repo.git

# Push
git push origin main

# Pull
git pull origin main

# Create branches
git checkout -b feature-branch
git push -u origin feature-branch
```

## Using the SDKs

### TypeScript

```bash
npm install @guts/sdk
```

```typescript
import { GutsClient } from '@guts/sdk';

const client = new GutsClient({
  baseUrl: 'https://api.guts.network',
  token: 'guts_xxx',
});

// List your repositories
const repos = await client.repos.list();
console.log(repos);
```

### Python

```bash
pip install guts-sdk
```

```python
from guts import GutsClient

client = GutsClient(
    base_url="https://api.guts.network",
    token="guts_xxx",
)

# List your repositories
repos = client.repos.list()
print(repos)
```

## Next Steps

- [Create a Pull Request](/developer/guides/first-pr)
- [Open an Issue](/developer/guides/first-issue)
- [Set up CI/CD](/developer/guides/ci-cd)
- [Migrate from GitHub](/developer/guides/migration)
- [API Reference](/developer/api/)

# Quickstart Guide

Get started with Guts in 5 minutes!

## Prerequisites

- Git installed
- A Guts account (or use a public node)

## Step 1: Install the CLI

```bash
# Using cargo
cargo install guts-cli

# Or download from releases
curl -sSL https://get.guts.network | sh
```

## Step 2: Create Your Identity

```bash
# Generate a new Ed25519 keypair
guts identity generate

# This creates ~/.guts/identity.json
```

## Step 3: Get a Token

Visit your Guts dashboard to create a personal access token, or use the CLI:

```bash
guts auth login
```

## Step 4: Clone a Repository

```bash
git clone https://guts.network/owner/repo.git
```

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

## Next Steps

- [Create a Pull Request](first-pr.md)
- [Open an Issue](first-issue.md)
- [Set up CI/CD](../guides/ci-cd.md)
- [Migrate from GitHub](../guides/migration.md)

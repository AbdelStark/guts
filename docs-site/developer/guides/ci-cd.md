# CI/CD Integration

Set up automated workflows with Guts CI/CD.

## Overview

Guts provides built-in CI/CD capabilities with:

- YAML-based workflow definitions
- Isolated job execution
- Artifact management
- Status checks integration
- Real-time log streaming

## Workflow Configuration

Create workflow files in `.guts/workflows/`:

```yaml
# .guts/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'

      - name: Install dependencies
        run: npm ci

      - name: Run tests
        run: npm test

      - name: Build
        run: npm run build
```

## Workflow Syntax

### Triggers

```yaml
on:
  # Push to specific branches
  push:
    branches: [main, develop]
    paths:
      - 'src/**'
      - 'package.json'

  # Pull requests
  pull_request:
    branches: [main]

  # Schedule (cron)
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

  # Manual trigger
  workflow_dispatch:
    inputs:
      environment:
        description: 'Deployment environment'
        required: true
        default: 'staging'
```

### Jobs

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm test

  build:
    needs: test  # Run after test job
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm run build

  deploy:
    needs: [test, build]  # Run after both jobs
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - run: ./deploy.sh
```

### Matrix Builds

```yaml
jobs:
  test:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        node: [18, 20, 22]
        os: [ubuntu-latest, macos-latest]
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: ${{ matrix.node }}
      - run: npm test
```

### Environment Variables

```yaml
jobs:
  build:
    runs-on: ubuntu-latest
    env:
      NODE_ENV: production
    steps:
      - run: echo $NODE_ENV
      - run: echo ${{ secrets.API_KEY }}
```

## Artifacts

### Upload Artifacts

```yaml
- name: Build
  run: npm run build

- name: Upload build artifacts
  uses: actions/upload-artifact@v4
  with:
    name: build-output
    path: dist/
```

### Download Artifacts

```yaml
- name: Download build artifacts
  uses: actions/download-artifact@v4
  with:
    name: build-output
    path: dist/
```

## Status Checks

Workflows automatically create status checks for pull requests:

```yaml
# Branch protection integrates with workflow status
jobs:
  required-check:
    runs-on: ubuntu-latest
    steps:
      - run: npm test
```

Configure in branch protection:

```bash
curl -X PUT "https://api.guts.network/api/repos/owner/repo/branches/main/protection" \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "required_status_checks": {
      "strict": true,
      "contexts": ["CI / test", "CI / build"]
    }
  }'
```

## Viewing Workflow Runs

### Via CLI

```bash
# List workflow runs
guts workflow runs owner/repo

# Get run details
guts workflow run owner/repo 123

# View logs
guts workflow logs owner/repo 123
```

### Via API

```bash
# List runs
curl https://api.guts.network/api/repos/owner/repo/actions/runs \
  -H "Authorization: Bearer guts_xxx"

# Get run
curl https://api.guts.network/api/repos/owner/repo/actions/runs/123 \
  -H "Authorization: Bearer guts_xxx"

# Get logs
curl https://api.guts.network/api/repos/owner/repo/actions/runs/123/logs \
  -H "Authorization: Bearer guts_xxx"
```

## Secrets

### Repository Secrets

```bash
# Create a secret
curl -X PUT https://api.guts.network/api/repos/owner/repo/actions/secrets/API_KEY \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{"value": "secret-value"}'
```

### Organization Secrets

```bash
# Create org secret
curl -X PUT https://api.guts.network/api/orgs/acme/actions/secrets/DEPLOY_TOKEN \
  -H "Authorization: Bearer guts_xxx" \
  -H "Content-Type: application/json" \
  -d '{
    "value": "secret-value",
    "visibility": "selected",
    "selected_repositories": ["repo1", "repo2"]
  }'
```

## Example Workflows

### Node.js

```yaml
name: Node.js CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      - run: npm ci
      - run: npm test
      - run: npm run lint
```

### Rust

```yaml
name: Rust CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: cargo clippy -- -D warnings
      - run: cargo test
```

### Docker Build

```yaml
name: Docker

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build image
        run: docker build -t myapp:${{ github.sha }} .

      - name: Push to registry
        run: |
          docker login -u ${{ secrets.DOCKER_USER }} -p ${{ secrets.DOCKER_PASS }}
          docker push myapp:${{ github.sha }}
```

### Release

```yaml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  release:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build
        run: npm run build

      - name: Create Release
        uses: actions/create-release@v1
        with:
          tag_name: ${{ github.ref_name }}
          release_name: Release ${{ github.ref_name }}
          draft: false
          prerelease: false

      - name: Upload assets
        run: |
          guts release upload ${{ github.ref_name }} dist/app-linux.tar.gz
          guts release upload ${{ github.ref_name }} dist/app-macos.tar.gz
```

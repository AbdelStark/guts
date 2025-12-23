# @guts/sdk

Official TypeScript SDK for the [Guts](https://github.com/AbdelStark/guts) decentralized code collaboration platform.

## Installation

```bash
npm install @guts/sdk
# or
yarn add @guts/sdk
# or
pnpm add @guts/sdk
```

## Quick Start

```typescript
import { GutsClient } from '@guts/sdk';

// Create a client instance
const client = new GutsClient({
  baseUrl: 'https://api.guts.network',
  token: 'guts_your_token_here', // optional
});

// List repositories
const repos = await client.repos.list();
console.log(repos.items);

// Get a specific repository
const repo = await client.repos.get('owner', 'repo-name');
console.log(repo);
```

## Features

- Full TypeScript support with comprehensive type definitions
- Promise-based API with async/await
- Repository management (create, list, update, delete)
- Issue tracking (create, update, close, comment)
- Pull request management (create, review, merge)
- Release management with asset uploads
- Organization and team management
- Webhook configuration
- Real-time event subscriptions
- Consensus status monitoring

## API Reference

### Repositories

```typescript
// List all repositories
const repos = await client.repos.list({ page: 1, per_page: 20 });

// Get a repository
const repo = await client.repos.get('owner', 'repo-name');

// Create a repository
const newRepo = await client.repos.create({
  name: 'my-new-repo',
  description: 'A new repository',
  private: false,
});

// Update a repository
const updated = await client.repos.update('owner', 'repo-name', {
  description: 'Updated description',
});

// Delete a repository
await client.repos.delete('owner', 'repo-name');
```

### Issues

```typescript
// List issues
const issues = await client.issues.list('owner', 'repo', { state: 'open' });

// Create an issue
const issue = await client.issues.create('owner', 'repo', {
  title: 'Bug report',
  body: 'Description of the bug',
  labels: ['bug'],
});

// Update an issue
await client.issues.update('owner', 'repo', 1, {
  title: 'Updated title',
});

// Close an issue
await client.issues.close('owner', 'repo', 1);

// Add a comment
await client.issues.createComment('owner', 'repo', 1, {
  body: 'This is a comment',
});
```

### Pull Requests

```typescript
// List pull requests
const prs = await client.pulls.list('owner', 'repo', { state: 'open' });

// Create a pull request
const pr = await client.pulls.create('owner', 'repo', {
  title: 'Feature: Add new feature',
  body: 'Description of changes',
  source_branch: 'feature-branch',
  target_branch: 'main',
});

// Create a review
await client.pulls.createReview('owner', 'repo', 1, {
  state: 'approve',
  body: 'LGTM!',
});

// Merge a pull request
await client.pulls.merge('owner', 'repo', 1, 'squash');
```

### Releases

```typescript
// List releases
const releases = await client.releases.list('owner', 'repo');

// Create a release
const release = await client.releases.create('owner', 'repo', {
  tag_name: 'v1.0.0',
  name: 'Version 1.0.0',
  body: 'Release notes here',
});

// Upload an asset
const file = new Blob(['binary content'], { type: 'application/octet-stream' });
await client.releases.uploadAsset(
  'owner',
  'repo',
  release.id,
  'app-linux-amd64',
  'application/octet-stream',
  await file.arrayBuffer()
);
```

### Organizations

```typescript
// List organizations
const orgs = await client.orgs.list();

// Get an organization
const org = await client.orgs.get('org-name');

// List teams
const teams = await client.orgs.listTeams('org-name');

// Create a team
const team = await client.orgs.createTeam('org-name', {
  name: 'developers',
  description: 'Development team',
});
```

### Consensus (Network Status)

```typescript
// Get consensus status
const status = await client.consensus.status();
console.log(`Block height: ${status.height}`);
console.log(`Synced: ${status.synced}`);

// List recent blocks
const blocks = await client.consensus.listBlocks(10);

// List validators
const validators = await client.consensus.listValidators();
```

### Real-time Events

```typescript
import { createEventSource, Channels } from '@guts/sdk';

// Subscribe to repository events
const eventSource = createEventSource({
  baseUrl: 'https://api.guts.network',
  token: 'guts_xxx',
  channel: Channels.repo('owner', 'repo'),
  onEvent: (event) => {
    console.log('Event:', event.type, event.payload);
  },
  onError: (error) => {
    console.error('Error:', error);
  },
});

// Close when done
eventSource.close();
```

## Error Handling

```typescript
import { GutsClient, GutsError } from '@guts/sdk';

try {
  const repo = await client.repos.get('owner', 'repo');
} catch (error) {
  if (error instanceof GutsError) {
    if (error.isNotFound()) {
      console.log('Repository not found');
    } else if (error.isUnauthorized()) {
      console.log('Invalid token');
    } else if (error.isRateLimited()) {
      console.log('Rate limited, try again later');
    } else {
      console.log(`Error ${error.status}: ${error.message}`);
    }
  }
}
```

## Configuration

```typescript
const client = new GutsClient({
  // Base URL of the Guts API
  baseUrl: 'https://api.guts.network',

  // Personal access token (optional)
  token: 'guts_xxx',

  // Request timeout in milliseconds (default: 30000)
  timeout: 60000,

  // Custom fetch implementation (optional)
  fetch: customFetch,
});
```

## License

MIT OR Apache-2.0

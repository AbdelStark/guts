# TypeScript SDK

The official TypeScript/JavaScript SDK for Guts.

## Installation

::: code-group

```bash [npm]
npm install @guts/sdk
```

```bash [yarn]
yarn add @guts/sdk
```

```bash [pnpm]
pnpm add @guts/sdk
```

:::

## Quick Start

```typescript
import { GutsClient } from '@guts/sdk';

// Create a client
const client = new GutsClient({
  baseUrl: 'https://api.guts.network',
  token: process.env.GUTS_TOKEN,
});

// List your repositories
const repos = await client.repos.list();
console.log(repos.items);
```

## Configuration

```typescript
import { GutsClient, GutsClientConfig } from '@guts/sdk';

const config: GutsClientConfig = {
  // Required: API base URL
  baseUrl: 'https://api.guts.network',

  // Required: Authentication token
  token: 'guts_xxx',

  // Optional: Request timeout (default: 30000ms)
  timeout: 30000,

  // Optional: Custom fetch implementation
  fetch: globalThis.fetch,

  // Optional: Retry configuration
  retry: {
    maxAttempts: 3,
    baseDelay: 1000,
    maxDelay: 30000,
  },
};

const client = new GutsClient(config);
```

## Repositories

```typescript
// List all repositories
const repos = await client.repos.list();

// Get a specific repository
const repo = await client.repos.get('owner', 'repo-name');

// Create a repository
const newRepo = await client.repos.create({
  name: 'my-new-repo',
  owner: 'myusername',
  description: 'A great project',
  private: false,
});

// Update a repository
await client.repos.update('owner', 'repo-name', {
  description: 'Updated description',
});

// Delete a repository
await client.repos.delete('owner', 'repo-name');
```

## Issues

```typescript
// List issues
const issues = await client.issues.list('owner', 'repo', {
  state: 'open',
  labels: ['bug'],
});

// Get an issue
const issue = await client.issues.get('owner', 'repo', 42);

// Create an issue
const newIssue = await client.issues.create('owner', 'repo', {
  title: 'Bug: Something is broken',
  body: 'When I click the button...',
  labels: ['bug'],
  assignees: ['developer'],
});

// Update an issue
await client.issues.update('owner', 'repo', 42, {
  state: 'closed',
});

// Add a comment
await client.issues.createComment('owner', 'repo', 42, {
  body: 'Fixed in PR #43',
});
```

## Pull Requests

```typescript
// List pull requests
const prs = await client.pulls.list('owner', 'repo', {
  state: 'open',
});

// Get a pull request
const pr = await client.pulls.get('owner', 'repo', 10);

// Create a pull request
const newPR = await client.pulls.create('owner', 'repo', {
  title: 'Add new feature',
  body: 'This PR implements...',
  source_branch: 'feature-branch',
  target_branch: 'main',
});

// Merge a pull request
await client.pulls.merge('owner', 'repo', 10, {
  merge_method: 'squash',
  commit_title: 'feat: Add new feature (#10)',
});

// Create a review
await client.pulls.createReview('owner', 'repo', 10, {
  body: 'LGTM!',
  event: 'APPROVE',
});
```

## Organizations

```typescript
// List organizations
const orgs = await client.orgs.list();

// Get an organization
const org = await client.orgs.get('acme');

// Create an organization
const newOrg = await client.orgs.create({
  name: 'acme',
  display_name: 'Acme Corp',
  description: 'Building the future',
});

// List teams
const teams = await client.orgs.listTeams('acme');

// Create a team
await client.orgs.createTeam('acme', {
  name: 'engineering',
  description: 'Engineering team',
  permission: 'write',
});
```

## Releases

```typescript
// List releases
const releases = await client.releases.list('owner', 'repo');

// Get latest release
const latest = await client.releases.getLatest('owner', 'repo');

// Create a release
const release = await client.releases.create('owner', 'repo', {
  tag_name: 'v1.0.0',
  name: 'Version 1.0.0',
  body: '## What\'s New\n\n- Feature 1\n- Feature 2',
});

// Upload asset
await client.releases.uploadAsset(release.id, {
  name: 'app-linux-amd64.tar.gz',
  data: fileBuffer,
  contentType: 'application/gzip',
});
```

## Consensus

```typescript
// Get consensus status
const status = await client.consensus.getStatus();
console.log(`Block height: ${status.block_height}`);

// List validators
const validators = await client.consensus.getValidators();

// Get recent blocks
const blocks = await client.consensus.listBlocks({ per_page: 10 });
```

## Error Handling

```typescript
import { GutsError, NotFoundError, RateLimitError } from '@guts/sdk';

try {
  const repo = await client.repos.get('owner', 'nonexistent');
} catch (error) {
  if (error instanceof NotFoundError) {
    console.log('Repository not found');
  } else if (error instanceof RateLimitError) {
    console.log(`Rate limited. Retry after: ${error.retryAfter}s`);
  } else if (error instanceof GutsError) {
    console.log(`API error: ${error.message}`);
    console.log(`Error code: ${error.code}`);
    console.log(`Status: ${error.status}`);
  } else {
    throw error;
  }
}
```

## Pagination

```typescript
// Automatic pagination with async iteration
for await (const repo of client.repos.listAll()) {
  console.log(repo.name);
}

// Manual pagination
let page = 1;
let hasMore = true;

while (hasMore) {
  const response = await client.repos.list({ page, per_page: 100 });
  for (const repo of response.items) {
    console.log(repo.name);
  }
  hasMore = page < response.total_pages;
  page++;
}
```

## Real-time Events

```typescript
// Subscribe to repository events
const subscription = client.realtime.subscribe('repo:owner/repo-name');

subscription.on('push', (event) => {
  console.log(`New push to ${event.ref}`);
});

subscription.on('issue', (event) => {
  console.log(`Issue ${event.action}: ${event.issue.title}`);
});

subscription.on('pull_request', (event) => {
  console.log(`PR ${event.action}: ${event.pull_request.title}`);
});

// Unsubscribe when done
subscription.close();
```

## TypeScript Support

The SDK is written in TypeScript and provides full type definitions:

```typescript
import type {
  Repository,
  Issue,
  PullRequest,
  Organization,
  Release,
  Validator,
  CreateIssueRequest,
  CreatePullRequestRequest,
} from '@guts/sdk';

// All responses are fully typed
const repo: Repository = await client.repos.get('owner', 'repo');

// Request types ensure correct parameters
const request: CreateIssueRequest = {
  title: 'Bug report',
  body: 'Description',
  labels: ['bug'],
};
```

## Node.js / Deno / Bun

The SDK works in all JavaScript runtimes:

```typescript
// Node.js (ESM)
import { GutsClient } from '@guts/sdk';

// Node.js (CommonJS)
const { GutsClient } = require('@guts/sdk');

// Deno
import { GutsClient } from 'npm:@guts/sdk';

// Bun
import { GutsClient } from '@guts/sdk';
```

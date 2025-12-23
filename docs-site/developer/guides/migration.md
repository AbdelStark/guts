# Migrate from GitHub

Migrate your repositories from GitHub to Guts with full history, issues, and pull requests.

## Quick Migration

### Using the CLI

```bash
# Install the migration tool
cargo install guts-migrate

# Migrate a repository
guts-migrate github \
  --repo owner/repo \
  --token $GITHUB_TOKEN \
  --guts-url https://api.guts.network \
  --guts-token $GUTS_TOKEN
```

### What Gets Migrated

| Content | Migrated |
|---------|----------|
| Git history | Yes |
| Branches | Yes |
| Tags | Yes |
| Issues | Yes |
| Pull requests | Yes |
| Issue comments | Yes |
| PR comments | Yes |
| Releases | Yes |
| Release assets | Yes |
| Labels | Yes |
| Wiki | Optional |

### Migration Options

```bash
guts-migrate github \
  --repo owner/repo \
  --token $GITHUB_TOKEN \
  --guts-url https://api.guts.network \
  --issues true \          # Migrate issues (default: true)
  --pull-requests true \   # Migrate PRs (default: true)
  --releases true \        # Migrate releases (default: true)
  --wiki true              # Migrate wiki (default: true)
```

## Using the TypeScript SDK

```typescript
import { GitHubMigrator, MigrationConfig, MigrationOptions } from '@guts/migrate';

const config: MigrationConfig = {
  source_repo: 'owner/repo',
  guts_url: 'https://api.guts.network',
  guts_token: 'guts_xxx',
};

const options: MigrationOptions = {
  migrate_issues: true,
  migrate_pull_requests: true,
  migrate_releases: true,
};

const migrator = new GitHubMigrator('github_token', config);
const report = await migrator.migrate(options);

report.printSummary();
```

## Using the Python SDK

```python
from guts.migrate import GitHubMigrator, MigrationConfig, MigrationOptions

config = MigrationConfig(
    source_repo="owner/repo",
    guts_url="https://api.guts.network",
    guts_token="guts_xxx",
)

options = MigrationOptions(
    migrate_issues=True,
    migrate_pull_requests=True,
    migrate_releases=True,
)

migrator = GitHubMigrator("github_token", config)
report = migrator.migrate(options)

report.print_summary()
```

## Verification

After migration, verify the data:

```bash
guts-migrate verify \
  --source https://github.com/owner/repo \
  --target owner/repo
```

This checks:
- Commit count matches
- Branch count matches
- Tag count matches
- Issue count matches
- PR count matches

## Post-Migration

### Update Remotes

```bash
cd your-repo
git remote set-url origin https://guts.network/owner/repo.git
```

### Set Up Redirect

You can set up a redirect from your old GitHub repository:

1. Add a prominent notice to the README
2. Archive the GitHub repository
3. Add repository description pointing to Guts

### Update CI/CD

If you're using GitHub Actions, adapt them to Guts workflows:

```yaml
# .guts/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: npm test
```

## Troubleshooting

### Rate Limit Exceeded

If you hit GitHub's rate limit:

```bash
guts-migrate github --repo owner/repo --wait-for-rate-limit
```

### Large Repositories

For repositories over 1GB:

```bash
# Use shallow clone first
guts-migrate github --repo owner/repo --shallow --depth 1

# Then fetch full history incrementally
guts-migrate github --repo owner/repo --incremental
```

### Private Repositories

Ensure your GitHub token has `repo` scope:

1. Go to GitHub Settings > Developer settings > Personal access tokens
2. Generate a new token with `repo` scope
3. Use this token for migration

## Migration Report

After migration, you'll receive a detailed report:

```
Migration Report
================
Source: github.com/owner/repo
Target: guts.network/owner/repo

Git Objects:
  Commits: 1,234 migrated
  Branches: 5 migrated
  Tags: 12 migrated

Collaboration:
  Issues: 45 migrated
  Pull Requests: 23 migrated
  Comments: 156 migrated

Releases:
  Releases: 8 migrated
  Assets: 24 migrated (156 MB)

Status: SUCCESS
Duration: 3m 42s
```

## Migrating Organizations

To migrate an entire organization:

```bash
guts-migrate github-org \
  --org acme-corp \
  --token $GITHUB_TOKEN \
  --guts-url https://api.guts.network \
  --guts-token $GUTS_TOKEN \
  --include-private
```

This will:
1. Create the organization on Guts
2. Migrate all repositories
3. Migrate teams and memberships
4. Migrate organization-level settings

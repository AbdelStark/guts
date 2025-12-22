# Milestone 13: User Adoption & Ecosystem

> **Status:** Planned
> **Target:** Q3 2025
> **Priority:** High

## Overview

Milestone 14 focuses on enabling mass adoption of Guts by providing migration tools, developer SDKs, IDE integrations, and a comprehensive developer experience. The goal is to make switching from GitHub to Guts as frictionless as possible while building an ecosystem of tools and integrations that make Guts a compelling platform for developers.

## Goals

1. **Migration Tools**: One-click migration from GitHub, GitLab, and Bitbucket
2. **Developer SDKs**: Official SDKs for TypeScript, Python, Go, and Rust
3. **IDE Integrations**: VS Code extension, JetBrains plugin
4. **Git Integration**: Native Git credential helper and SSH support
5. **Developer Documentation**: Comprehensive API docs, tutorials, and examples
6. **Community Platform**: Forums, Discord, and support infrastructure
7. **Ecosystem Growth**: Third-party integrations, CI/CD adapters

## Migration Tools

### Architecture

```
tools/migration/
├── guts-migrate/           # CLI migration tool
│   ├── src/
│   │   ├── main.rs
│   │   ├── github.rs       # GitHub migration
│   │   ├── gitlab.rs       # GitLab migration
│   │   ├── bitbucket.rs    # Bitbucket migration
│   │   ├── progress.rs     # Progress reporting
│   │   └── verify.rs       # Migration verification
│   └── Cargo.toml
├── web/                    # Web-based migration wizard
│   ├── src/
│   │   ├── pages/
│   │   │   ├── index.tsx
│   │   │   ├── github.tsx
│   │   │   └── progress.tsx
│   │   └── components/
│   └── package.json
└── docs/
    ├── github-migration.md
    ├── gitlab-migration.md
    └── troubleshooting.md
```

## Detailed Implementation

### Phase 1: GitHub Migration Tool

#### 1.1 CLI Migration Tool

```rust
// tools/migration/guts-migrate/src/main.rs

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "guts-migrate")]
#[command(about = "Migrate repositories to Guts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Migrate from GitHub
    Github {
        /// GitHub repository (owner/repo)
        #[arg(short, long)]
        repo: String,

        /// GitHub personal access token
        #[arg(short, long, env = "GITHUB_TOKEN")]
        token: String,

        /// Guts node URL
        #[arg(long, default_value = "https://api.guts.network")]
        guts_url: String,

        /// Include issues
        #[arg(long, default_value = "true")]
        issues: bool,

        /// Include pull requests
        #[arg(long, default_value = "true")]
        pull_requests: bool,

        /// Include releases
        #[arg(long, default_value = "true")]
        releases: bool,

        /// Include wiki
        #[arg(long, default_value = "true")]
        wiki: bool,
    },
    /// Migrate from GitLab
    Gitlab {
        // Similar options
    },
    /// Migrate from Bitbucket
    Bitbucket {
        // Similar options
    },
    /// Verify migration
    Verify {
        /// Source repository URL
        source: String,
        /// Guts repository
        target: String,
    },
}
```

#### 1.2 GitHub Migration Implementation

```rust
// tools/migration/guts-migrate/src/github.rs

use octocrab::Octocrab;
use indicatif::{ProgressBar, ProgressStyle};

pub struct GitHubMigration {
    github: Octocrab,
    guts: GutsClient,
    config: MigrationConfig,
}

#[derive(Clone)]
pub struct MigrationConfig {
    pub repo: String,
    pub issues: bool,
    pub pull_requests: bool,
    pub releases: bool,
    pub wiki: bool,
}

impl GitHubMigration {
    pub async fn new(token: &str, guts_url: &str) -> Result<Self> {
        let github = Octocrab::builder()
            .personal_token(token.to_string())
            .build()?;

        let guts = GutsClient::new(guts_url)?;

        Ok(Self { github, guts, config: Default::default() })
    }

    /// Run complete migration
    pub async fn migrate(&self, config: MigrationConfig) -> Result<MigrationReport> {
        let mut report = MigrationReport::new();

        // Step 1: Get repository info
        println!("Fetching repository information...");
        let (owner, name) = parse_repo(&config.repo)?;
        let gh_repo = self.github.repos(owner, name).get().await?;

        // Step 2: Create Guts repository
        println!("Creating repository on Guts...");
        let guts_repo = self.guts.create_repo(&CreateRepoRequest {
            name: gh_repo.name.clone(),
            description: gh_repo.description.clone(),
            private: gh_repo.private.unwrap_or(false),
        }).await?;
        report.repo_created = true;

        // Step 3: Mirror Git data
        println!("Mirroring Git repository...");
        self.mirror_git(&gh_repo, &guts_repo).await?;
        report.git_mirrored = true;

        // Step 4: Migrate issues
        if config.issues {
            println!("Migrating issues...");
            let count = self.migrate_issues(&gh_repo, &guts_repo).await?;
            report.issues_migrated = count;
        }

        // Step 5: Migrate pull requests
        if config.pull_requests {
            println!("Migrating pull requests...");
            let count = self.migrate_pull_requests(&gh_repo, &guts_repo).await?;
            report.prs_migrated = count;
        }

        // Step 6: Migrate releases
        if config.releases {
            println!("Migrating releases...");
            let count = self.migrate_releases(&gh_repo, &guts_repo).await?;
            report.releases_migrated = count;
        }

        // Step 7: Migrate wiki
        if config.wiki && gh_repo.has_wiki.unwrap_or(false) {
            println!("Migrating wiki...");
            self.migrate_wiki(&gh_repo, &guts_repo).await?;
            report.wiki_migrated = true;
        }

        // Step 8: Set up redirect (optional)
        println!("Setting up GitHub redirect...");
        self.setup_redirect(&gh_repo, &guts_repo).await?;

        Ok(report)
    }

    async fn mirror_git(&self, source: &Repository, target: &GutsRepo) -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let clone_path = temp_dir.path().join("repo");

        // Clone with all branches and tags
        let output = Command::new("git")
            .args(["clone", "--mirror", &source.clone_url.unwrap(), clone_path.to_str().unwrap()])
            .output()?;

        if !output.status.success() {
            return Err(Error::GitCloneFailed(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        // Push to Guts
        let output = Command::new("git")
            .current_dir(&clone_path)
            .args(["push", "--mirror", &target.clone_url])
            .output()?;

        if !output.status.success() {
            return Err(Error::GitPushFailed(String::from_utf8_lossy(&output.stderr).to_string()));
        }

        Ok(())
    }

    async fn migrate_issues(&self, source: &Repository, target: &GutsRepo) -> Result<usize> {
        let issues = self.github.issues(&source.owner.login, &source.name)
            .list()
            .state(octocrab::params::State::All)
            .send()
            .await?;

        let pb = ProgressBar::new(issues.items.len() as u64);
        pb.set_style(ProgressStyle::default_bar()
            .template("{msg} [{bar:40.cyan/blue}] {pos}/{len}")
            .unwrap());
        pb.set_message("Migrating issues");

        let mut count = 0;
        for issue in issues.items {
            // Create issue on Guts
            let guts_issue = self.guts.create_issue(&target.key, &CreateIssueRequest {
                title: issue.title,
                body: self.rewrite_content(&issue.body.unwrap_or_default()),
                labels: issue.labels.iter().map(|l| l.name.clone()).collect(),
                assignees: issue.assignees.iter().map(|a| self.map_user(&a.login)).collect(),
            }).await?;

            // Migrate comments
            let comments = self.github.issues(&source.owner.login, &source.name)
                .list_comments(issue.number)
                .send()
                .await?;

            for comment in comments.items {
                self.guts.create_comment(&target.key, guts_issue.number, &CreateCommentRequest {
                    body: self.rewrite_content(&comment.body.unwrap_or_default()),
                }).await?;
            }

            // Close if closed on GitHub
            if issue.state == IssueState::Closed {
                self.guts.close_issue(&target.key, guts_issue.number).await?;
            }

            pb.inc(1);
            count += 1;
        }

        pb.finish_with_message("Issues migrated");
        Ok(count)
    }

    async fn migrate_pull_requests(&self, source: &Repository, target: &GutsRepo) -> Result<usize> {
        let prs = self.github.pulls(&source.owner.login, &source.name)
            .list()
            .state(octocrab::params::State::All)
            .send()
            .await?;

        let mut count = 0;
        for pr in prs.items {
            // Create PR on Guts
            let guts_pr = self.guts.create_pull_request(&target.key, &CreatePullRequest {
                title: pr.title.unwrap_or_default(),
                body: self.rewrite_content(&pr.body.unwrap_or_default()),
                source_branch: pr.head.ref_field,
                target_branch: pr.base.ref_field,
            }).await?;

            // Migrate reviews
            let reviews = self.github.pulls(&source.owner.login, &source.name)
                .list_reviews(pr.number)
                .await?;

            for review in reviews.items {
                self.guts.create_review(&target.key, guts_pr.number, &CreateReview {
                    body: review.body.unwrap_or_default(),
                    state: self.map_review_state(&review.state),
                }).await?;
            }

            count += 1;
        }

        Ok(count)
    }

    /// Rewrite content to update links and references
    fn rewrite_content(&self, content: &str) -> String {
        // Rewrite GitHub URLs to Guts URLs
        let content = content.replace(
            &format!("https://github.com/{}", self.config.repo),
            &format!("https://guts.network/{}", self.config.repo)
        );

        // Rewrite user mentions
        // @github-user -> @guts-user (if mapped)

        // Rewrite issue references
        // #123 -> guts#123

        content
    }
}

#[derive(Default)]
pub struct MigrationReport {
    pub repo_created: bool,
    pub git_mirrored: bool,
    pub issues_migrated: usize,
    pub prs_migrated: usize,
    pub releases_migrated: usize,
    pub wiki_migrated: bool,
    pub errors: Vec<String>,
}

impl MigrationReport {
    pub fn print_summary(&self) {
        println!("\n=== Migration Summary ===");
        println!("Repository created: {}", if self.repo_created { "✓" } else { "✗" });
        println!("Git data mirrored: {}", if self.git_mirrored { "✓" } else { "✗" });
        println!("Issues migrated: {}", self.issues_migrated);
        println!("Pull requests migrated: {}", self.prs_migrated);
        println!("Releases migrated: {}", self.releases_migrated);
        println!("Wiki migrated: {}", if self.wiki_migrated { "✓" } else { "N/A" });

        if !self.errors.is_empty() {
            println!("\nErrors:");
            for error in &self.errors {
                println!("  - {}", error);
            }
        }
    }
}
```

### Phase 2: Developer SDKs

#### 2.1 TypeScript SDK

```typescript
// packages/guts-sdk/src/index.ts

export class GutsClient {
  private baseUrl: string;
  private token?: string;

  constructor(options: GutsClientOptions) {
    this.baseUrl = options.baseUrl || 'https://api.guts.network';
    this.token = options.token;
  }

  // Repository operations
  repos = {
    list: () => this.get<Repository[]>('/api/repos'),
    get: (owner: string, name: string) =>
      this.get<Repository>(`/api/repos/${owner}/${name}`),
    create: (data: CreateRepoRequest) =>
      this.post<Repository>('/api/repos', data),
    delete: (owner: string, name: string) =>
      this.delete(`/api/repos/${owner}/${name}`),
  };

  // Pull request operations
  pulls = {
    list: (owner: string, repo: string) =>
      this.get<PullRequest[]>(`/api/repos/${owner}/${repo}/pulls`),
    get: (owner: string, repo: string, number: number) =>
      this.get<PullRequest>(`/api/repos/${owner}/${repo}/pulls/${number}`),
    create: (owner: string, repo: string, data: CreatePullRequest) =>
      this.post<PullRequest>(`/api/repos/${owner}/${repo}/pulls`, data),
    merge: (owner: string, repo: string, number: number) =>
      this.post(`/api/repos/${owner}/${repo}/pulls/${number}/merge`, {}),
  };

  // Issue operations
  issues = {
    list: (owner: string, repo: string, options?: ListIssuesOptions) =>
      this.get<Issue[]>(`/api/repos/${owner}/${repo}/issues`, options),
    get: (owner: string, repo: string, number: number) =>
      this.get<Issue>(`/api/repos/${owner}/${repo}/issues/${number}`),
    create: (owner: string, repo: string, data: CreateIssue) =>
      this.post<Issue>(`/api/repos/${owner}/${repo}/issues`, data),
    update: (owner: string, repo: string, number: number, data: UpdateIssue) =>
      this.patch<Issue>(`/api/repos/${owner}/${repo}/issues/${number}`, data),
  };

  // WebSocket for real-time updates
  subscribe(channel: string): EventSource {
    const url = new URL(`/ws/subscribe`, this.baseUrl);
    url.searchParams.set('channel', channel);
    if (this.token) {
      url.searchParams.set('token', this.token);
    }
    return new EventSource(url.toString());
  }

  private async get<T>(path: string, params?: Record<string, any>): Promise<T> {
    const url = new URL(path, this.baseUrl);
    if (params) {
      Object.entries(params).forEach(([key, value]) =>
        url.searchParams.set(key, String(value))
      );
    }

    const response = await fetch(url.toString(), {
      headers: this.headers(),
    });

    if (!response.ok) {
      throw new GutsError(response.status, await response.text());
    }

    return response.json();
  }

  private async post<T>(path: string, data: any): Promise<T> {
    const response = await fetch(new URL(path, this.baseUrl).toString(), {
      method: 'POST',
      headers: this.headers(),
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      throw new GutsError(response.status, await response.text());
    }

    return response.json();
  }

  private headers(): Record<string, string> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };
    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`;
    }
    return headers;
  }
}

// React hooks
export function useRepository(owner: string, name: string) {
  const [repo, setRepo] = useState<Repository | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const client = new GutsClient({});
    client.repos.get(owner, name)
      .then(setRepo)
      .catch(setError)
      .finally(() => setLoading(false));
  }, [owner, name]);

  return { repo, loading, error };
}

export function useIssues(owner: string, repo: string) {
  // Similar implementation
}
```

#### 2.2 Python SDK

```python
# packages/guts-python/guts/__init__.py

from dataclasses import dataclass
from typing import Optional, List, Iterator
import httpx

@dataclass
class GutsConfig:
    base_url: str = "https://api.guts.network"
    token: Optional[str] = None

class GutsClient:
    def __init__(self, config: Optional[GutsConfig] = None):
        self.config = config or GutsConfig()
        self._client = httpx.Client(
            base_url=self.config.base_url,
            headers=self._headers(),
        )

    def _headers(self) -> dict:
        headers = {"Content-Type": "application/json"}
        if self.config.token:
            headers["Authorization"] = f"Bearer {self.config.token}"
        return headers

    # Repository operations
    def list_repos(self) -> List["Repository"]:
        response = self._client.get("/api/repos")
        response.raise_for_status()
        return [Repository(**r) for r in response.json()]

    def get_repo(self, owner: str, name: str) -> "Repository":
        response = self._client.get(f"/api/repos/{owner}/{name}")
        response.raise_for_status()
        return Repository(**response.json())

    def create_repo(
        self,
        name: str,
        description: Optional[str] = None,
        private: bool = False,
    ) -> "Repository":
        response = self._client.post(
            "/api/repos",
            json={"name": name, "description": description, "private": private},
        )
        response.raise_for_status()
        return Repository(**response.json())

    # Pull request operations
    def list_pulls(
        self, owner: str, repo: str, state: str = "open"
    ) -> List["PullRequest"]:
        response = self._client.get(
            f"/api/repos/{owner}/{repo}/pulls",
            params={"state": state},
        )
        response.raise_for_status()
        return [PullRequest(**pr) for pr in response.json()]

    def create_pull(
        self,
        owner: str,
        repo: str,
        title: str,
        source_branch: str,
        target_branch: str,
        body: Optional[str] = None,
    ) -> "PullRequest":
        response = self._client.post(
            f"/api/repos/{owner}/{repo}/pulls",
            json={
                "title": title,
                "body": body,
                "source_branch": source_branch,
                "target_branch": target_branch,
            },
        )
        response.raise_for_status()
        return PullRequest(**response.json())

    # Issue operations
    def list_issues(
        self, owner: str, repo: str, state: str = "open"
    ) -> List["Issue"]:
        response = self._client.get(
            f"/api/repos/{owner}/{repo}/issues",
            params={"state": state},
        )
        response.raise_for_status()
        return [Issue(**issue) for issue in response.json()]

    def create_issue(
        self,
        owner: str,
        repo: str,
        title: str,
        body: Optional[str] = None,
        labels: Optional[List[str]] = None,
    ) -> "Issue":
        response = self._client.post(
            f"/api/repos/{owner}/{repo}/issues",
            json={"title": title, "body": body, "labels": labels or []},
        )
        response.raise_for_status()
        return Issue(**response.json())

    # Context manager support
    def __enter__(self):
        return self

    def __exit__(self, *args):
        self._client.close()

# CLI convenience
def cli():
    """Command-line interface for Guts."""
    import argparse

    parser = argparse.ArgumentParser(description="Guts CLI")
    parser.add_argument("--token", help="API token")
    # Add subcommands...

    args = parser.parse_args()
    # Handle commands...
```

### Phase 3: IDE Integrations

#### 3.1 VS Code Extension

```typescript
// extensions/vscode-guts/src/extension.ts

import * as vscode from 'vscode';
import { GutsClient } from 'guts-sdk';

export function activate(context: vscode.ExtensionContext) {
  const client = new GutsClient({
    token: vscode.workspace.getConfiguration('guts').get('token'),
  });

  // Repository explorer
  const repoProvider = new GutsRepoProvider(client);
  vscode.window.registerTreeDataProvider('gutsRepos', repoProvider);

  // Pull request view
  const prProvider = new GutsPullRequestProvider(client);
  vscode.window.registerTreeDataProvider('gutsPullRequests', prProvider);

  // Issue view
  const issueProvider = new GutsIssueProvider(client);
  vscode.window.registerTreeDataProvider('gutsIssues', issueProvider);

  // Commands
  context.subscriptions.push(
    vscode.commands.registerCommand('guts.cloneRepo', async () => {
      const repo = await vscode.window.showInputBox({
        prompt: 'Enter repository (owner/name)',
        placeHolder: 'owner/repo',
      });

      if (repo) {
        const uri = vscode.Uri.parse(`guts://${repo}`);
        await vscode.commands.executeCommand('git.clone', uri);
      }
    }),

    vscode.commands.registerCommand('guts.createPullRequest', async () => {
      const repo = await getCurrentRepo();
      if (!repo) {
        vscode.window.showErrorMessage('No Guts repository found');
        return;
      }

      const title = await vscode.window.showInputBox({
        prompt: 'Pull request title',
      });

      if (title) {
        const pr = await client.pulls.create(repo.owner, repo.name, {
          title,
          source_branch: await getCurrentBranch(),
          target_branch: 'main',
        });

        vscode.window.showInformationMessage(
          `Created PR #${pr.number}`,
          'Open in Browser'
        ).then((action) => {
          if (action === 'Open in Browser') {
            vscode.env.openExternal(vscode.Uri.parse(pr.html_url));
          }
        });
      }
    }),

    vscode.commands.registerCommand('guts.createIssue', async () => {
      // Similar implementation
    }),
  );

  // Status bar
  const statusBar = vscode.window.createStatusBarItem(
    vscode.StatusBarAlignment.Left,
    100
  );
  statusBar.text = '$(git-branch) Guts';
  statusBar.command = 'guts.showMenu';
  statusBar.show();

  // Git credential helper registration
  registerGitCredentialHelper(context);
}

class GutsRepoProvider implements vscode.TreeDataProvider<RepoItem> {
  private _onDidChangeTreeData = new vscode.EventEmitter<RepoItem | undefined>();
  readonly onDidChangeTreeData = this._onDidChangeTreeData.event;

  constructor(private client: GutsClient) {}

  async getChildren(element?: RepoItem): Promise<RepoItem[]> {
    if (!element) {
      const repos = await this.client.repos.list();
      return repos.map((r) => new RepoItem(r));
    }
    return [];
  }

  getTreeItem(element: RepoItem): vscode.TreeItem {
    return element;
  }
}
```

#### 3.2 Git Credential Helper

```rust
// tools/git-credential-guts/src/main.rs

use std::io::{self, BufRead, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let operation = args.get(1).map(|s| s.as_str()).unwrap_or("");

    match operation {
        "get" => handle_get()?,
        "store" => handle_store()?,
        "erase" => handle_erase()?,
        _ => {
            eprintln!("Usage: git-credential-guts <get|store|erase>");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn handle_get() -> Result<(), Box<dyn std::error::Error>> {
    let mut protocol = String::new();
    let mut host = String::new();

    // Read input from Git
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.is_empty() {
            break;
        }

        let mut parts = line.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");

        match key {
            "protocol" => protocol = value.to_string(),
            "host" => host = value.to_string(),
            _ => {}
        }
    }

    // Check if this is a Guts host
    if !is_guts_host(&host) {
        return Ok(());
    }

    // Get token from secure storage
    if let Some(token) = get_stored_token(&host)? {
        println!("protocol={}", protocol);
        println!("host={}", host);
        println!("username=token");
        println!("password={}", token);
    }

    Ok(())
}

fn handle_store() -> Result<(), Box<dyn std::error::Error>> {
    let mut host = String::new();
    let mut password = String::new();

    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.is_empty() {
            break;
        }

        let mut parts = line.splitn(2, '=');
        let key = parts.next().unwrap_or("");
        let value = parts.next().unwrap_or("");

        match key {
            "host" => host = value.to_string(),
            "password" => password = value.to_string(),
            _ => {}
        }
    }

    if is_guts_host(&host) && !password.is_empty() {
        store_token(&host, &password)?;
    }

    Ok(())
}

fn is_guts_host(host: &str) -> bool {
    host.ends_with(".guts.network") || host == "guts.network" || host == "localhost"
}

fn get_stored_token(host: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Use system keychain (keyring crate)
    let entry = keyring::Entry::new("git-credential-guts", host)?;
    match entry.get_password() {
        Ok(token) => Ok(Some(token)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

fn store_token(host: &str, token: &str) -> Result<(), Box<dyn std::error::Error>> {
    let entry = keyring::Entry::new("git-credential-guts", host)?;
    entry.set_password(token)?;
    Ok(())
}
```

### Phase 4: SSH Support

#### 4.1 SSH Key Management

```rust
// crates/guts-compat/src/ssh.rs

use ssh_key::{PublicKey, Algorithm};

pub struct SshKeyManager {
    keys: HashMap<UserId, Vec<SshKey>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SshKey {
    pub id: Uuid,
    pub user_id: UserId,
    pub title: String,
    pub key_type: KeyType,
    pub public_key: String,
    pub fingerprint: String,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum KeyType {
    Ed25519,
    EcdsaSha2Nistp256,
    EcdsaSha2Nistp384,
    Rsa,
}

impl SshKeyManager {
    /// Add SSH key for user
    pub fn add_key(&mut self, user_id: UserId, title: String, public_key: String) -> Result<SshKey> {
        // Parse and validate key
        let parsed = PublicKey::from_openssh(&public_key)?;

        let key_type = match parsed.algorithm() {
            Algorithm::Ed25519 => KeyType::Ed25519,
            Algorithm::EcdsaSha2NistP256 => KeyType::EcdsaSha2Nistp256,
            Algorithm::EcdsaSha2NistP384 => KeyType::EcdsaSha2Nistp384,
            Algorithm::Rsa { .. } => KeyType::Rsa,
            _ => return Err(Error::UnsupportedKeyType),
        };

        let fingerprint = parsed.fingerprint(ssh_key::HashAlg::Sha256).to_string();

        let key = SshKey {
            id: Uuid::new_v4(),
            user_id,
            title,
            key_type,
            public_key,
            fingerprint,
            created_at: Utc::now(),
            last_used: None,
        };

        self.keys.entry(user_id).or_default().push(key.clone());

        Ok(key)
    }

    /// Authenticate by SSH key
    pub fn authenticate(&self, fingerprint: &str) -> Option<UserId> {
        for (user_id, keys) in &self.keys {
            for key in keys {
                if key.fingerprint == fingerprint {
                    return Some(*user_id);
                }
            }
        }
        None
    }
}
```

#### 4.2 SSH Server

```rust
// crates/guts-node/src/ssh_server.rs

use russh::*;
use russh_keys::*;

pub struct GutsSshServer {
    git_service: Arc<GitService>,
    auth: Arc<AuthService>,
}

impl server::Server for GutsSshServer {
    type Handler = GutsSshHandler;

    fn new_client(&mut self, _: Option<SocketAddr>) -> Self::Handler {
        GutsSshHandler {
            git_service: self.git_service.clone(),
            auth: self.auth.clone(),
            authenticated_user: None,
        }
    }
}

pub struct GutsSshHandler {
    git_service: Arc<GitService>,
    auth: Arc<AuthService>,
    authenticated_user: Option<UserId>,
}

impl server::Handler for GutsSshHandler {
    type Error = anyhow::Error;

    async fn auth_publickey(
        &mut self,
        user: &str,
        public_key: &key::PublicKey,
    ) -> Result<server::Auth, Self::Error> {
        let fingerprint = public_key.fingerprint();

        if let Some(user_id) = self.auth.authenticate_ssh(&fingerprint).await? {
            self.authenticated_user = Some(user_id);
            Ok(server::Auth::Accept)
        } else {
            Ok(server::Auth::Reject)
        }
    }

    async fn exec_request(
        &mut self,
        channel: ChannelId,
        command: &[u8],
        session: &mut Session,
    ) -> Result<(), Self::Error> {
        let command = String::from_utf8_lossy(command);

        // Parse git command
        if command.starts_with("git-upload-pack") || command.starts_with("git-receive-pack") {
            let repo_path = extract_repo_path(&command)?;

            // Check permissions
            let user = self.authenticated_user.ok_or(Error::NotAuthenticated)?;
            self.auth.check_permission(user, &repo_path, Permission::Read).await?;

            // Handle git operation
            self.handle_git_command(channel, &command, session).await?;
        }

        Ok(())
    }
}
```

### Phase 5: Developer Documentation

#### 5.1 Documentation Site Structure

```
docs/developer/
├── index.md                  # Developer guide overview
├── quickstart/
│   ├── first-repo.md         # Create first repository
│   ├── first-pr.md           # Create first PR
│   └── first-issue.md        # Create first issue
├── guides/
│   ├── authentication.md     # Auth methods
│   ├── webhooks.md           # Setting up webhooks
│   ├── ci-cd.md              # CI/CD integration
│   ├── migration.md          # Migration from GitHub
│   └── best-practices.md     # Best practices
├── api/
│   ├── overview.md           # API overview
│   ├── authentication.md     # API authentication
│   ├── repositories.md       # Repository endpoints
│   ├── pull-requests.md      # PR endpoints
│   ├── issues.md             # Issue endpoints
│   ├── webhooks.md           # Webhook events
│   └── rate-limits.md        # Rate limiting
├── sdks/
│   ├── typescript.md         # TypeScript SDK
│   ├── python.md             # Python SDK
│   ├── go.md                 # Go SDK
│   └── rust.md               # Rust SDK
├── integrations/
│   ├── vscode.md             # VS Code extension
│   ├── jetbrains.md          # JetBrains plugin
│   ├── github-actions.md     # GitHub Actions adapter
│   └── gitlab-ci.md          # GitLab CI adapter
└── reference/
    ├── git-protocol.md       # Git protocol details
    ├── openapi.yaml          # OpenAPI spec
    └── errors.md             # Error codes
```

### Phase 6: Community Platform

#### 6.1 Community Infrastructure

```yaml
# Community platform components
community:
  forum:
    platform: discourse
    url: https://forum.guts.network
    categories:
      - General Discussion
      - Help & Support
      - Feature Requests
      - Show & Tell
      - Development

  discord:
    invite: https://discord.gg/guts
    channels:
      - "#general"
      - "#help"
      - "#development"
      - "#operators"
      - "#governance"

  documentation:
    platform: docusaurus
    url: https://docs.guts.network

  status:
    platform: statuspage
    url: https://status.guts.network
```

## Implementation Plan

### Phase 1: Migration Tools (Week 1-4)
- [ ] Implement GitHub migration CLI
- [ ] Add GitLab migration support
- [ ] Add Bitbucket migration support
- [ ] Create web-based migration wizard
- [ ] Write migration documentation

### Phase 2: SDKs (Week 4-7)
- [ ] Develop TypeScript SDK
- [ ] Develop Python SDK
- [ ] Develop Go SDK
- [ ] Update Rust client library
- [ ] Create SDK documentation

### Phase 3: IDE Integrations (Week 7-10)
- [ ] Build VS Code extension
- [ ] Build JetBrains plugin
- [ ] Implement Git credential helper
- [ ] Create integration guides

### Phase 4: SSH Support (Week 10-11)
- [ ] Implement SSH key management
- [ ] Build SSH server
- [ ] Test with standard Git clients
- [ ] Document SSH setup

### Phase 5: Documentation (Week 11-13)
- [ ] Write developer quickstart
- [ ] Create API documentation
- [ ] Write integration guides
- [ ] Create video tutorials

### Phase 6: Community (Week 13-14)
- [ ] Set up forum
- [ ] Create Discord server
- [ ] Launch documentation site
- [ ] Establish support processes

## Success Criteria

- [ ] One-click migration from GitHub works for 95% of repositories
- [ ] SDKs available for TypeScript, Python, Go, Rust
- [ ] VS Code extension published with 100+ installs
- [ ] Git credential helper works seamlessly
- [ ] SSH clone/push works with standard Git
- [ ] Documentation site live with 50+ pages
- [ ] Community forum active with 100+ members
- [ ] 10+ repositories migrated from GitHub

## Dependencies

- GitHub API access for migration
- VS Code extension marketplace account
- JetBrains plugin marketplace account
- Documentation hosting (Vercel, Netlify)
- Discourse hosting
- Discord server setup

## References

- [GitHub REST API](https://docs.github.com/en/rest)
- [VS Code Extension API](https://code.visualstudio.com/api)
- [Git Credential Helper Protocol](https://git-scm.com/docs/git-credential)
- [russh Library](https://github.com/warp-tech/russh)
- [Docusaurus](https://docusaurus.io/)

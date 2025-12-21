# Milestone 7: CI/CD Integration

> **Status:** In Progress
> **Started:** 2025-12-21

## Overview

Milestone 7 implements decentralized CI/CD pipelines, enabling automated build, test, and deployment workflows. This completes the developer workflow experience by allowing repositories to define and execute continuous integration pipelines that run across the Guts network.

## Goals

1. **Workflow Configuration**: YAML-based pipeline definitions (`.guts/workflows/*.yml`)
2. **Job Execution**: Isolated job execution with step-by-step processing
3. **Status Checks**: Integration with branch protection and PR workflows
4. **Artifact Management**: Store and retrieve build artifacts
5. **Real-time Logs**: Stream build logs to connected clients
6. **Distributed Execution**: Jobs can be executed by any node in the network

## Architecture

### New Crate: `guts-ci`

```
crates/guts-ci/
├── src/
│   ├── lib.rs           # Public API
│   ├── error.rs         # Error types
│   ├── workflow.rs      # Workflow definition parsing
│   ├── pipeline.rs      # Pipeline types
│   ├── job.rs           # Job definition and execution
│   ├── step.rs          # Individual step types
│   ├── run.rs           # Workflow run tracking
│   ├── artifact.rs      # Artifact storage
│   ├── status.rs        # Status check integration
│   ├── trigger.rs       # Trigger types (push, PR, manual)
│   ├── executor.rs      # Job execution engine
│   └── store.rs         # CI/CD data storage
└── Cargo.toml
```

### Technology Choices

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Config Format | YAML (serde_yaml) | Standard CI/CD config format |
| Isolation | Tokio subprocess | Process isolation for commands |
| Storage | Content-addressed | Artifacts stored by hash |
| Streaming | Channel + WebSocket | Real-time log streaming |

### Workflow Configuration

Configuration files are stored in `.guts/workflows/*.yml`:

```yaml
# .guts/workflows/ci.yml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

env:
  RUST_BACKTRACE: "1"

jobs:
  build:
    name: Build
    runs-on: default
    steps:
      - name: Checkout
        uses: checkout

      - name: Build
        run: cargo build --workspace

      - name: Test
        run: cargo test --workspace

  lint:
    name: Lint
    runs-on: default
    steps:
      - name: Checkout
        uses: checkout

      - name: Format check
        run: cargo fmt --all -- --check

      - name: Clippy
        run: cargo clippy --workspace -- -D warnings

  security:
    name: Security Audit
    runs-on: default
    needs: [build]
    steps:
      - name: Checkout
        uses: checkout

      - name: Audit
        run: cargo audit
```

### Core Types

#### Workflow

```rust
/// A complete workflow definition
pub struct Workflow {
    pub id: WorkflowId,
    pub name: String,
    pub repo_key: String,
    pub path: String,           // e.g., ".guts/workflows/ci.yml"
    pub triggers: Vec<Trigger>,
    pub env: HashMap<String, String>,
    pub jobs: HashMap<String, JobDefinition>,
    pub created_at: u64,
    pub updated_at: u64,
}

/// What triggers a workflow
pub enum Trigger {
    Push { branches: Vec<String>, paths: Option<Vec<String>> },
    PullRequest { branches: Vec<String>, types: Vec<PrEventType> },
    Schedule { cron: String },
    Manual,
    WorkflowDispatch { inputs: HashMap<String, InputDefinition> },
}
```

#### Job Definition

```rust
/// A job within a workflow
pub struct JobDefinition {
    pub name: String,
    pub runs_on: String,
    pub needs: Vec<String>,     // Dependencies on other jobs
    pub env: HashMap<String, String>,
    pub steps: Vec<Step>,
    pub timeout_minutes: u32,
    pub continue_on_error: bool,
}

/// A single step in a job
pub enum Step {
    Run {
        name: Option<String>,
        run: String,
        working_directory: Option<String>,
        env: HashMap<String, String>,
        shell: Option<String>,
    },
    Uses {
        name: Option<String>,
        uses: String,           // e.g., "checkout", "cache"
        with: HashMap<String, String>,
    },
}
```

#### Workflow Run

```rust
/// A single execution of a workflow
pub struct WorkflowRun {
    pub id: RunId,
    pub workflow_id: WorkflowId,
    pub repo_key: String,
    pub number: u32,
    pub status: RunStatus,
    pub conclusion: Option<Conclusion>,
    pub trigger: TriggerContext,
    pub head_sha: String,
    pub head_branch: Option<String>,
    pub jobs: HashMap<String, JobRun>,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
    pub created_at: u64,
}

pub enum RunStatus {
    Queued,
    InProgress,
    Completed,
}

pub enum Conclusion {
    Success,
    Failure,
    Cancelled,
    Skipped,
    TimedOut,
}
```

#### Job Run

```rust
/// Execution state of a job
pub struct JobRun {
    pub id: JobRunId,
    pub name: String,
    pub status: RunStatus,
    pub conclusion: Option<Conclusion>,
    pub steps: Vec<StepRun>,
    pub runner: Option<String>,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
}

/// Execution state of a step
pub struct StepRun {
    pub number: u32,
    pub name: String,
    pub status: RunStatus,
    pub conclusion: Option<Conclusion>,
    pub started_at: Option<u64>,
    pub completed_at: Option<u64>,
}
```

#### Artifacts

```rust
/// Build artifact metadata
pub struct Artifact {
    pub id: ArtifactId,
    pub run_id: RunId,
    pub name: String,
    pub size_bytes: u64,
    pub content_hash: String,   // SHA-256
    pub expires_at: Option<u64>,
    pub created_at: u64,
}
```

#### Status Checks

```rust
/// Status check for a commit
pub struct StatusCheck {
    pub id: StatusCheckId,
    pub repo_key: String,
    pub sha: String,
    pub context: String,        // e.g., "CI / Build"
    pub state: CheckState,
    pub description: Option<String>,
    pub target_url: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}

pub enum CheckState {
    Pending,
    Success,
    Failure,
    Error,
}
```

### Data Flow

```
Push/PR Event
     │
     ▼
┌─────────────────────────────────┐
│         Trigger Matcher         │
│  (match workflow triggers)      │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│       Workflow Run Created      │
│  (queued, status check pending) │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│          Job Scheduler          │
│  (resolve dependencies, queue)  │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│          Job Executor           │
│  ┌───────────────────────────┐  │
│  │     Step 1: Checkout      │  │
│  │     Step 2: Build         │  │
│  │     Step 3: Test          │  │
│  └───────────────────────────┘  │
└────────────────┬────────────────┘
                 │
                 ▼ (logs via WebSocket)
┌─────────────────────────────────┐
│         Log Streaming           │
│  (real-time to connected UI)    │
└────────────────┬────────────────┘
                 │
                 ▼
┌─────────────────────────────────┐
│       Status Check Updated      │
│  (success/failure/error)        │
└─────────────────────────────────┘
```

## Implementation Plan

### Phase 1: Core Types & Storage

1. [x] Create `guts-ci` crate structure
2. [x] Define core types (Workflow, Job, Step, Run)
3. [x] Implement YAML workflow parsing
4. [x] Create workflow storage
5. [x] Add validation for workflow configurations

### Phase 2: Workflow Runs

1. [x] Implement WorkflowRun tracking
2. [x] Create trigger matching logic
3. [x] Implement job dependency resolution
4. [x] Add run status management
5. [x] Create run storage and retrieval

### Phase 3: Job Execution

1. [x] Implement step executor
2. [x] Add environment variable handling
3. [x] Create log capture and streaming
4. [x] Implement timeout handling
5. [x] Add artifact upload/download

### Phase 4: Status Checks Integration

1. [x] Create StatusCheck type
2. [x] Integrate with guts-auth branch protection
3. [x] Auto-create checks on workflow trigger
4. [x] Update checks on job completion
5. [x] Block PR merge on failed required checks

### Phase 5: API Endpoints

1. [x] GET/POST `/api/repos/{owner}/{name}/workflows`
2. [x] GET `/api/repos/{owner}/{name}/workflows/{workflow_id}`
3. [x] GET/POST `/api/repos/{owner}/{name}/runs`
4. [x] GET `/api/repos/{owner}/{name}/runs/{run_id}`
5. [x] POST `/api/repos/{owner}/{name}/runs/{run_id}/cancel`
6. [x] GET `/api/repos/{owner}/{name}/runs/{run_id}/jobs`
7. [x] GET `/api/repos/{owner}/{name}/runs/{run_id}/jobs/{job_id}/logs`
8. [x] GET/POST `/api/repos/{owner}/{name}/artifacts`
9. [x] GET `/api/repos/{owner}/{name}/commits/{sha}/status`
10. [x] POST `/api/repos/{owner}/{name}/commits/{sha}/statuses`

### Phase 6: Web UI

1. [x] Workflow list page (`/{owner}/{repo}/actions`)
2. [x] Workflow detail page (`/{owner}/{repo}/actions/{workflow}`)
3. [x] Run detail page (`/{owner}/{repo}/actions/runs/{run_id}`)
4. [x] Job log viewer with ANSI color support
5. [x] Status badges for README
6. [x] Real-time log streaming via WebSocket

### Phase 7: CLI Commands

1. [x] `guts workflow list` - List workflows
2. [x] `guts workflow run` - Trigger workflow manually
3. [x] `guts run list` - List workflow runs
4. [x] `guts run logs` - View run logs
5. [x] `guts run cancel` - Cancel a running workflow

## API Reference

### Workflow Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/repos/{owner}/{name}/workflows` | List workflows |
| POST | `/api/repos/{owner}/{name}/workflows` | Create/update workflow |
| GET | `/api/repos/{owner}/{name}/workflows/{id}` | Get workflow details |
| DELETE | `/api/repos/{owner}/{name}/workflows/{id}` | Delete workflow |

### Run Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/repos/{owner}/{name}/runs` | List runs |
| POST | `/api/repos/{owner}/{name}/runs` | Trigger manual run |
| GET | `/api/repos/{owner}/{name}/runs/{id}` | Get run details |
| POST | `/api/repos/{owner}/{name}/runs/{id}/cancel` | Cancel run |
| POST | `/api/repos/{owner}/{name}/runs/{id}/rerun` | Re-run workflow |

### Job Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/repos/{owner}/{name}/runs/{id}/jobs` | List jobs in run |
| GET | `/api/repos/{owner}/{name}/runs/{id}/jobs/{job}` | Get job details |
| GET | `/api/repos/{owner}/{name}/runs/{id}/jobs/{job}/logs` | Get job logs |

### Artifact Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/repos/{owner}/{name}/runs/{id}/artifacts` | List artifacts |
| POST | `/api/repos/{owner}/{name}/runs/{id}/artifacts` | Upload artifact |
| GET | `/api/repos/{owner}/{name}/runs/{id}/artifacts/{name}` | Download artifact |
| DELETE | `/api/repos/{owner}/{name}/runs/{id}/artifacts/{name}` | Delete artifact |

### Status Check Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/repos/{owner}/{name}/commits/{sha}/status` | Get combined status |
| GET | `/api/repos/{owner}/{name}/commits/{sha}/statuses` | List all statuses |
| POST | `/api/repos/{owner}/{name}/commits/{sha}/statuses` | Create status check |

## WebSocket Events

New event types for CI/CD:

| Event | Description |
|-------|-------------|
| `workflow_run.requested` | Workflow run created |
| `workflow_run.in_progress` | Workflow run started |
| `workflow_run.completed` | Workflow run finished |
| `workflow_job.queued` | Job queued |
| `workflow_job.in_progress` | Job started |
| `workflow_job.completed` | Job finished |
| `check_run.created` | Status check created |
| `check_run.completed` | Status check completed |

## Built-in Actions

The following actions are available by default:

| Action | Description |
|--------|-------------|
| `checkout` | Checkout repository at current commit |
| `cache` | Cache dependencies between runs |
| `upload-artifact` | Upload build artifacts |
| `download-artifact` | Download artifacts from other jobs |

## Success Criteria

- [x] Workflows parse correctly from YAML
- [x] Jobs execute in correct dependency order
- [x] Status checks integrate with branch protection
- [x] Logs stream in real-time via WebSocket
- [x] Artifacts can be uploaded and downloaded
- [x] Web UI shows workflow runs and logs
- [x] CLI can trigger and monitor workflows
- [x] Failed required checks block PR merge

## Security Considerations

1. **Sandboxing**: Jobs run in isolated environments
2. **Secrets Management**: Encrypted secret storage (future)
3. **Timeout Limits**: Prevent runaway jobs
4. **Resource Limits**: Cap CPU/memory per job
5. **Network Isolation**: Restrict network access in jobs
6. **Audit Logging**: Log all CI/CD operations

## Performance Considerations

1. **Parallel Jobs**: Jobs without dependencies run in parallel
2. **Caching**: Cache build dependencies
3. **Incremental Builds**: Only rebuild changed components
4. **Log Compression**: Compress stored logs
5. **Artifact Deduplication**: Content-addressed storage

## Future Enhancements

These features are out of scope for Milestone 7 but planned:

1. **Secrets Management**: Encrypted secrets for workflows
2. **Self-hosted Runners**: Custom execution environments
3. **Matrix Builds**: Parallel builds across configurations
4. **Reusable Workflows**: Import workflows from other repos
5. **Workflow Templates**: Pre-built workflow templates
6. **Cost Attribution**: Track compute usage per repository

## Dependencies

- `guts-auth`: Branch protection, permissions
- `guts-collaboration`: PR status integration
- `guts-realtime`: WebSocket event streaming
- `guts-storage`: Artifact storage
- `serde_yaml`: YAML parsing
- `tokio`: Async execution

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [GitLab CI/CD](https://docs.gitlab.com/ee/ci/)
- [Tekton Pipelines](https://tekton.dev/)
- [Woodpecker CI](https://woodpecker-ci.org/)

# Milestone 5: Ecosystem - Web Gateway

> **Status:** In Progress
> **Started:** 2025-12-21

## Overview

Milestone 5 implements ecosystem features that enhance the user experience and enable broader adoption. The primary focus is on a **Web Gateway** that provides browser-based access to repositories, making Guts accessible to users who prefer a web interface over CLI.

## Goals

1. **Web Gateway**: Browser-based repository access with modern UI
2. **Repository Browser**: View files, commits, branches in the browser
3. **Collaboration UI**: View PRs, Issues, and discussions in the browser
4. **Search**: Search repositories, code, issues, and PRs
5. **API Documentation**: Interactive API documentation (OpenAPI/Swagger)

## Architecture

### New Crate: `guts-web`

```
crates/guts-web/
├── src/
│   ├── lib.rs           # Public API
│   ├── routes.rs        # Web route handlers
│   ├── templates/       # HTML templates (Askama)
│   │   ├── base.html
│   │   ├── index.html
│   │   ├── repo/
│   │   │   ├── list.html
│   │   │   ├── view.html
│   │   │   ├── tree.html
│   │   │   ├── blob.html
│   │   │   └── commits.html
│   │   ├── pr/
│   │   │   ├── list.html
│   │   │   └── view.html
│   │   ├── issue/
│   │   │   ├── list.html
│   │   │   └── view.html
│   │   └── org/
│   │       ├── list.html
│   │       └── view.html
│   ├── static/          # Static assets (CSS, JS)
│   │   ├── style.css
│   │   └── app.js
│   └── error.rs         # Error handling
└── Cargo.toml
```

### Technology Choices

| Component | Technology | Rationale |
|-----------|------------|-----------|
| Templates | Askama | Type-safe, compile-time templates |
| CSS | Tailwind CSS (CDN) | Utility-first, no build step |
| Syntax Highlighting | highlight.js | Client-side code highlighting |
| Markdown | pulldown-cmark | Fast, safe Markdown rendering |

### Route Structure

```
/                           # Landing page
/explore                    # Explore repositories
/search                     # Search (repos, code, issues)

/{owner}                    # User/Org profile
/{owner}/{repo}             # Repository home (README)
/{owner}/{repo}/tree/{ref}  # File tree browser
/{owner}/{repo}/blob/{ref}/{path}  # File viewer
/{owner}/{repo}/commits/{ref}      # Commit history
/{owner}/{repo}/commit/{sha}       # Single commit
/{owner}/{repo}/branches    # Branch list
/{owner}/{repo}/tags        # Tag list

/{owner}/{repo}/pulls       # PR list
/{owner}/{repo}/pull/{num}  # PR details
/{owner}/{repo}/issues      # Issue list
/{owner}/{repo}/issues/{num}# Issue details

/{owner}/{repo}/settings    # Repo settings (auth required)
/orgs/{org}                 # Organization page
/orgs/{org}/teams           # Teams list

/api/docs                   # Interactive API documentation
```

### Page Components

#### Repository Home
- README rendering (Markdown)
- File tree preview
- Branch selector
- Clone URLs (HTTP)
- Stats (commits, contributors, stars)

#### File Browser
- Directory tree navigation
- File content with syntax highlighting
- Breadcrumb navigation
- Branch/tag selector
- "Raw" and "History" links

#### Commit History
- Commit list with author, date, message
- Commit diff viewer
- File changes summary

#### Pull Request View
- PR description (Markdown)
- Conversation thread
- Commits included
- Files changed with diff
- Review status
- Merge button (for admins)

#### Issue View
- Issue description (Markdown)
- Comment thread
- Labels and assignees
- Status (open/closed)
- Close/reopen actions

### Data Flow

```
Browser Request
       │
       ▼
┌─────────────┐
│  guts-node  │
│   (Axum)    │
└──────┬──────┘
       │
       ├─────────────────────────┐
       │                         │
       ▼                         ▼
┌─────────────┐          ┌─────────────┐
│  guts-web   │          │   REST API  │
│  (HTML UI)  │          │   (JSON)    │
└──────┬──────┘          └─────────────┘
       │
       ▼
┌─────────────┐
│   Askama    │
│  Templates  │
└─────────────┘
```

## Implementation Plan

### Phase 1: Foundation

1. Create `guts-web` crate structure
2. Set up Askama template engine
3. Create base template with navigation
4. Add static asset serving (CSS/JS)
5. Implement landing page

### Phase 2: Repository Browsing

1. Repository list page
2. Repository home page (README)
3. File tree browser
4. File content viewer with syntax highlighting
5. Commit history page
6. Single commit diff view

### Phase 3: Collaboration Views

1. Pull request list page
2. Pull request detail page
3. Issue list page
4. Issue detail page
5. Comment rendering

### Phase 4: Organization Views

1. User profile page
2. Organization page
3. Team list page

### Phase 5: Search & Discovery

1. Repository search
2. Code search
3. Issue/PR search
4. Advanced filters

### Phase 6: API Documentation

1. OpenAPI spec generation
2. Swagger UI integration
3. Interactive API explorer

## API Integration

The web gateway reuses existing API infrastructure:

```rust
// Web routes call internal API handlers
async fn repo_home(
    State(state): State<AppState>,
    Path((owner, repo)): Path<(String, String)>,
) -> impl IntoResponse {
    let repo_key = format!("{}/{}", owner, repo);

    // Reuse existing repo store
    let repo = state.repos.get(&repo_key)?;

    // Render template
    let template = RepoHomeTemplate {
        repo: &repo,
        readme: render_readme(&repo),
    };

    Html(template.render()?)
}
```

## Success Criteria

- [ ] Landing page with repository explorer
- [ ] Repository file browser with syntax highlighting
- [ ] Commit history and diff viewer
- [ ] Pull request and issue views
- [ ] Organization and team pages
- [ ] Code search functionality
- [ ] Responsive design (mobile-friendly)
- [ ] No JavaScript required for core functionality

## Dependencies

- `askama`: Template engine
- `pulldown-cmark`: Markdown rendering
- `syntect`: Server-side syntax highlighting (optional)
- `guts-storage`: Access git objects
- `guts-collaboration`: PR/Issue data
- `guts-auth`: Permission checking

## Future Considerations

These features are out of scope for Milestone 5 but planned:

1. **CI/CD Integration**: Decentralized build pipelines
2. **Package Registry**: Decentralized package hosting
3. **Federation**: Inter-network repository bridging
4. **Real-time Updates**: WebSocket for live updates
5. **Dark Mode**: Theme switching
6. **Keyboard Shortcuts**: Vim-style navigation
7. **Notifications**: In-browser notifications

## References

- [GitHub Web Interface](https://github.com)
- [GitLab Web Interface](https://gitlab.com)
- [Gitea](https://gitea.io) - Lightweight self-hosted
- [Askama Documentation](https://djc.github.io/askama/)

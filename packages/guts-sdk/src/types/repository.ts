/**
 * Repository types.
 */

export interface Repository {
  /** Repository key (owner/name) */
  key: string;
  /** Repository name */
  name: string;
  /** Repository owner */
  owner: string;
  /** Repository description */
  description?: string;
  /** Whether the repository is private */
  private: boolean;
  /** Default branch name */
  default_branch: string;
  /** Git clone URL */
  clone_url: string;
  /** Web URL for the repository */
  html_url: string;
  /** Number of open issues */
  open_issues_count: number;
  /** Number of open pull requests */
  open_pull_requests_count: number;
  /** Creation timestamp */
  created_at: string;
  /** Last update timestamp */
  updated_at: string;
  /** Last push timestamp */
  pushed_at?: string;
}

export interface CreateRepositoryRequest {
  /** Repository name */
  name: string;
  /** Repository description */
  description?: string;
  /** Whether the repository is private */
  private?: boolean;
  /** Initialize with a README */
  auto_init?: boolean;
  /** License template name */
  license_template?: string;
  /** .gitignore template name */
  gitignore_template?: string;
}

export interface UpdateRepositoryRequest {
  /** Repository name */
  name?: string;
  /** Repository description */
  description?: string;
  /** Whether the repository is private */
  private?: boolean;
  /** Default branch name */
  default_branch?: string;
}

/**
 * @guts/sdk - Official TypeScript SDK for Guts
 *
 * A type-safe client for interacting with the Guts decentralized code collaboration platform.
 *
 * @example
 * ```typescript
 * import { GutsClient } from '@guts/sdk';
 *
 * const client = new GutsClient({
 *   baseUrl: 'https://api.guts.network',
 *   token: 'guts_xxx',
 * });
 *
 * // List repositories
 * const repos = await client.repos.list();
 *
 * // Create an issue
 * const issue = await client.issues.create('owner', 'repo', {
 *   title: 'Bug report',
 *   body: 'Description of the bug',
 * });
 * ```
 */

export { GutsClient } from './client';
export type { GutsClientOptions } from './client';

// Types
export type {
  Repository,
  CreateRepositoryRequest,
  UpdateRepositoryRequest,
} from './types/repository';

export type {
  Issue,
  IssueState,
  CreateIssueRequest,
  UpdateIssueRequest,
  ListIssuesOptions,
} from './types/issue';

export type {
  PullRequest,
  PullRequestState,
  CreatePullRequestRequest,
  UpdatePullRequestRequest,
  ListPullRequestsOptions,
  MergeMethod,
} from './types/pull-request';

export type {
  Comment,
  CreateCommentRequest,
  UpdateCommentRequest,
} from './types/comment';

export type {
  Review,
  ReviewState,
  CreateReviewRequest,
} from './types/review';

export type {
  Release,
  ReleaseAsset,
  CreateReleaseRequest,
  UpdateReleaseRequest,
} from './types/release';

export type {
  Label,
  CreateLabelRequest,
  UpdateLabelRequest,
} from './types/label';

export type {
  User,
  UserProfile,
} from './types/user';

export type {
  Organization,
  Team,
  TeamMember,
  OrganizationMember,
} from './types/organization';

export type {
  Webhook,
  WebhookEvent,
  CreateWebhookRequest,
} from './types/webhook';

export type {
  ConsensusStatus,
  Block,
  Validator,
} from './types/consensus';

export type {
  PaginatedResponse,
  PaginationParams,
} from './types/pagination';

export type {
  GutsError,
  ApiError,
} from './types/error';

// Re-export utility functions
export { createEventSource } from './realtime';

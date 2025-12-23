/**
 * Release types.
 */

export interface Release {
  /** Release ID */
  id: string;
  /** Tag name */
  tag_name: string;
  /** Release name */
  name: string;
  /** Release body/notes */
  body?: string;
  /** Whether this is a draft release */
  draft: boolean;
  /** Whether this is a prerelease */
  prerelease: boolean;
  /** Release author */
  author: string;
  /** Target commitish (branch/tag/sha) */
  target_commitish: string;
  /** Assets attached to the release */
  assets: ReleaseAsset[];
  /** Creation timestamp */
  created_at: string;
  /** Publication timestamp */
  published_at?: string;
}

export interface ReleaseAsset {
  /** Asset ID */
  id: string;
  /** Asset name */
  name: string;
  /** Asset content type */
  content_type: string;
  /** Asset size in bytes */
  size: number;
  /** Download count */
  download_count: number;
  /** Download URL */
  browser_download_url: string;
  /** Upload timestamp */
  created_at: string;
  /** Uploader username */
  uploader: string;
}

export interface CreateReleaseRequest {
  /** Tag name */
  tag_name: string;
  /** Release name */
  name?: string;
  /** Release body/notes */
  body?: string;
  /** Target commitish (branch/tag/sha) */
  target_commitish?: string;
  /** Whether this is a draft release */
  draft?: boolean;
  /** Whether this is a prerelease */
  prerelease?: boolean;
  /** Auto-generate release notes */
  generate_release_notes?: boolean;
}

export interface UpdateReleaseRequest {
  /** Tag name */
  tag_name?: string;
  /** Release name */
  name?: string;
  /** Release body/notes */
  body?: string;
  /** Target commitish (branch/tag/sha) */
  target_commitish?: string;
  /** Whether this is a draft release */
  draft?: boolean;
  /** Whether this is a prerelease */
  prerelease?: boolean;
}

/**
 * User types.
 */

export interface User {
  /** User ID */
  id: string;
  /** Username */
  username: string;
  /** Display name */
  display_name?: string;
  /** Avatar URL */
  avatar_url?: string;
  /** User bio */
  bio?: string;
  /** Public key for verification */
  public_key: string;
  /** Account creation timestamp */
  created_at: string;
}

export interface UserProfile extends User {
  /** User's email (only visible to self) */
  email?: string;
  /** Number of public repositories */
  public_repos: number;
  /** Number of followers */
  followers: number;
  /** Number of following */
  following: number;
}

/**
 * Organization types.
 */

export type OrganizationRole = 'owner' | 'admin' | 'member';
export type TeamPermission = 'read' | 'write' | 'admin';

export interface Organization {
  /** Organization ID */
  id: string;
  /** Organization slug */
  slug: string;
  /** Organization display name */
  display_name?: string;
  /** Organization description */
  description?: string;
  /** Avatar URL */
  avatar_url?: string;
  /** Number of members */
  members_count: number;
  /** Number of repositories */
  repos_count: number;
  /** Creation timestamp */
  created_at: string;
}

export interface OrganizationMember {
  /** User information */
  user: {
    id: string;
    username: string;
    avatar_url?: string;
  };
  /** Role in the organization */
  role: OrganizationRole;
  /** Joined timestamp */
  joined_at: string;
}

export interface Team {
  /** Team ID */
  id: string;
  /** Team slug */
  slug: string;
  /** Team name */
  name: string;
  /** Team description */
  description?: string;
  /** Default permission for repositories */
  permission: TeamPermission;
  /** Number of members */
  members_count: number;
  /** Number of repositories */
  repos_count: number;
  /** Creation timestamp */
  created_at: string;
}

export interface TeamMember {
  /** User information */
  user: {
    id: string;
    username: string;
    avatar_url?: string;
  };
  /** Role in the team */
  role: 'maintainer' | 'member';
  /** Joined timestamp */
  joined_at: string;
}

/**
 * Webhook types.
 */

export type WebhookEvent =
  | 'push'
  | 'pull_request'
  | 'pull_request_review'
  | 'issues'
  | 'issue_comment'
  | 'create'
  | 'delete'
  | 'release'
  | 'fork'
  | 'star';

export interface Webhook {
  /** Webhook ID */
  id: string;
  /** Webhook URL */
  url: string;
  /** Events that trigger the webhook */
  events: WebhookEvent[];
  /** Whether the webhook is active */
  active: boolean;
  /** Secret for payload signing */
  secret?: string;
  /** Content type for payload */
  content_type: 'json' | 'form';
  /** Creation timestamp */
  created_at: string;
  /** Last update timestamp */
  updated_at: string;
  /** Last delivery timestamp */
  last_delivery_at?: string;
}

export interface CreateWebhookRequest {
  /** Webhook URL */
  url: string;
  /** Events that trigger the webhook */
  events: WebhookEvent[];
  /** Whether the webhook is active */
  active?: boolean;
  /** Secret for payload signing */
  secret?: string;
  /** Content type for payload */
  content_type?: 'json' | 'form';
}

export interface WebhookDelivery {
  /** Delivery ID */
  id: string;
  /** Event type */
  event: WebhookEvent;
  /** HTTP status code */
  status_code: number;
  /** Whether delivery was successful */
  success: boolean;
  /** Duration in milliseconds */
  duration_ms: number;
  /** Delivery timestamp */
  delivered_at: string;
  /** Request headers */
  request_headers: Record<string, string>;
  /** Request payload */
  request_payload: unknown;
  /** Response headers */
  response_headers?: Record<string, string>;
  /** Response body */
  response_body?: string;
}

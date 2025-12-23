/**
 * Real-time event subscription utilities.
 */

export interface RealtimeEvent {
  /** Event type */
  type: string;
  /** Event channel */
  channel: string;
  /** Event payload */
  payload: unknown;
  /** Event timestamp */
  timestamp: string;
}

export interface EventSourceOptions {
  /** Base URL of the Guts API */
  baseUrl: string;
  /** Authentication token */
  token?: string;
  /** Channel to subscribe to */
  channel: string;
  /** Callback for received events */
  onEvent: (event: RealtimeEvent) => void;
  /** Callback for errors */
  onError?: (error: Error) => void;
  /** Callback for connection open */
  onOpen?: () => void;
  /** Callback for connection close */
  onClose?: () => void;
}

/**
 * Create an EventSource for real-time updates.
 *
 * @example
 * ```typescript
 * const eventSource = createEventSource({
 *   baseUrl: 'https://api.guts.network',
 *   token: 'guts_xxx',
 *   channel: 'repo:owner/repo',
 *   onEvent: (event) => {
 *     console.log('Received event:', event);
 *   },
 * });
 *
 * // Later, close the connection
 * eventSource.close();
 * ```
 */
export function createEventSource(options: EventSourceOptions): EventSource {
  const url = new URL(`${options.baseUrl}/api/events`);
  url.searchParams.set('channel', options.channel);

  if (options.token) {
    url.searchParams.set('token', options.token);
  }

  const eventSource = new EventSource(url.toString());

  eventSource.onmessage = (event) => {
    try {
      const data = JSON.parse(event.data) as RealtimeEvent;
      options.onEvent(data);
    } catch (error) {
      options.onError?.(error as Error);
    }
  };

  eventSource.onerror = (event) => {
    options.onError?.(new Error('EventSource connection error'));
  };

  eventSource.onopen = () => {
    options.onOpen?.();
  };

  return eventSource;
}

/**
 * Channel types for subscription.
 */
export const Channels = {
  /** Subscribe to repository events */
  repo: (owner: string, name: string) => `repo:${owner}/${name}`,

  /** Subscribe to user events */
  user: (username: string) => `user:${username}`,

  /** Subscribe to organization events */
  org: (org: string) => `org:${org}`,

  /** Subscribe to all events (requires admin) */
  all: () => 'all',
} as const;

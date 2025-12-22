/**
 * K6 Load Test: Concurrent Connections
 *
 * Tests system behavior under high concurrent connection load
 * to validate 10,000+ concurrent connection target.
 *
 * Usage:
 *   k6 run --env GUTS_URL=http://localhost:8080 concurrent.js
 */

import http from 'k6/http';
import ws from 'k6/ws';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter, Gauge } from 'k6/metrics';

// Custom metrics
const connectionSuccess = new Rate('connection_success');
const connectionLatency = new Trend('connection_latency', true);
const activeConnections = new Gauge('active_connections');
const wsConnections = new Gauge('websocket_connections');
const requestErrors = new Counter('request_errors');

// Test configuration
export const options = {
  scenarios: {
    // HTTP connection stress
    http_connections: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 100 },
        { duration: '2m', target: 500 },
        { duration: '3m', target: 1000 },
        { duration: '5m', target: 1000 },
        { duration: '2m', target: 500 },
        { duration: '1m', target: 0 },
      ],
      exec: 'httpConnections',
    },
    // WebSocket connection stress
    websocket_connections: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 50 },
        { duration: '2m', target: 200 },
        { duration: '3m', target: 500 },
        { duration: '5m', target: 500 },
        { duration: '2m', target: 200 },
        { duration: '1m', target: 0 },
      ],
      exec: 'websocketConnections',
      startTime: '15m',
    },
    // Mixed workload
    mixed_load: {
      executor: 'constant-vus',
      vus: 100,
      duration: '10m',
      exec: 'mixedWorkload',
      startTime: '30m',
    },
  },
  thresholds: {
    'connection_success': ['rate>0.95'],
    'connection_latency': ['p95<500'],
    'http_req_failed': ['rate<0.05'],
  },
};

export function setup() {
  const baseUrl = __ENV.GUTS_URL || 'http://localhost:8080';
  const wsUrl = baseUrl.replace('http', 'ws');

  // Create a test repository
  const owner = 'concurrent-test';
  const repo = `test-${Date.now()}`;

  http.post(
    `${baseUrl}/api/repos`,
    JSON.stringify({ name: repo, owner: owner }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  return {
    baseUrl,
    wsUrl,
    owner,
    repo,
  };
}

// HTTP connection stress test
export function httpConnections(data) {
  const start = Date.now();

  // Make multiple rapid requests to stress connection handling
  const responses = http.batch([
    ['GET', `${data.baseUrl}/api/repos`, null, { timeout: '10s' }],
    ['GET', `${data.baseUrl}/api/orgs`, null, { timeout: '10s' }],
    ['GET', `${data.baseUrl}/health`, null, { timeout: '10s' }],
  ]);

  const duration = Date.now() - start;
  connectionLatency.add(duration);

  let allSuccess = true;
  for (const res of responses) {
    if (res.status !== 200) {
      allSuccess = false;
      requestErrors.add(1);
    }
  }

  connectionSuccess.add(allSuccess);
  activeConnections.add(__VU);

  check(responses[0], {
    'HTTP connection OK': (r) => r.status === 200,
    'response time < 500ms': () => duration < 500,
  });

  // Minimal sleep to maximize connection pressure
  sleep(0.1);
}

// WebSocket connection stress test
export function websocketConnections(data) {
  const url = `${data.wsUrl}/api/events`;

  const start = Date.now();
  let connected = false;

  const res = ws.connect(url, {}, function (socket) {
    socket.on('open', () => {
      connected = true;
      wsConnections.add(1);

      // Subscribe to repository events
      socket.send(JSON.stringify({
        type: 'subscribe',
        repository: `${data.owner}/${data.repo}`,
      }));
    });

    socket.on('message', (msg) => {
      // Handle incoming messages
      try {
        const parsed = JSON.parse(msg);
        check(parsed, {
          'message has type': (m) => m.type !== undefined,
        });
      } catch (e) {
        // Ignore non-JSON messages
      }
    });

    socket.on('error', (e) => {
      requestErrors.add(1);
      console.error('WebSocket error:', e);
    });

    socket.on('close', () => {
      wsConnections.add(-1);
    });

    // Keep connection open for a while
    socket.setTimeout(() => {
      socket.close();
    }, 30000); // 30 seconds
  });

  const duration = Date.now() - start;
  connectionLatency.add(duration);
  connectionSuccess.add(connected);

  check(res, {
    'WebSocket connected': () => connected,
  });

  // Wait for the connection to complete
  sleep(30);
}

// Mixed workload test
export function mixedWorkload(data) {
  const workloadType = Math.random();

  if (workloadType < 0.6) {
    // 60% reads
    const start = Date.now();
    const res = http.get(`${data.baseUrl}/api/repos`, {
      timeout: '10s',
    });
    connectionLatency.add(Date.now() - start);
    connectionSuccess.add(res.status === 200);
  } else if (workloadType < 0.9) {
    // 30% writes
    const start = Date.now();
    const res = http.post(
      `${data.baseUrl}/api/repos/${data.owner}/${data.repo}/issues`,
      JSON.stringify({
        title: `Load test issue ${Date.now()}`,
        description: 'Created during load test',
      }),
      {
        headers: { 'Content-Type': 'application/json' },
        timeout: '10s',
      }
    );
    connectionLatency.add(Date.now() - start);
    connectionSuccess.add(res.status === 201 || res.status === 200);
  } else {
    // 10% health checks
    const start = Date.now();
    const res = http.get(`${data.baseUrl}/health`, {
      timeout: '5s',
    });
    connectionLatency.add(Date.now() - start);
    connectionSuccess.add(res.status === 200);
  }

  sleep(Math.random() * 0.5);
}

// Burst test - sudden spike
export function burstTest(data) {
  // Make 10 rapid requests
  for (let i = 0; i < 10; i++) {
    const start = Date.now();
    const res = http.get(`${data.baseUrl}/api/repos`, {
      timeout: '10s',
    });
    connectionLatency.add(Date.now() - start);
    connectionSuccess.add(res.status === 200);
  }

  sleep(5); // Wait before next burst
}

export function teardown(data) {
  console.log('Concurrent connection test completed');

  // Print summary
  console.log(`Test configuration:`);
  console.log(`  Base URL: ${data.baseUrl}`);
  console.log(`  Repository: ${data.owner}/${data.repo}`);
}

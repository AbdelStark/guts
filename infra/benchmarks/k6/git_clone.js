/**
 * K6 Load Test: Git Clone/Fetch Operations
 *
 * Tests git-upload-pack endpoint under load to validate
 * clone throughput targets (> 10 MB/s).
 *
 * Usage:
 *   k6 run --env GUTS_URL=http://localhost:8080 git_clone.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
const cloneLatency = new Trend('git_clone_latency', true);
const cloneThroughput = new Trend('git_clone_throughput_mbps');
const cloneSuccess = new Rate('git_clone_success');
const cloneErrors = new Counter('git_clone_errors');
const bytesReceived = new Counter('git_bytes_received');

// Test configuration
export const options = {
  scenarios: {
    // Steady load test
    steady_load: {
      executor: 'constant-vus',
      vus: 20,
      duration: '5m',
    },
    // Spike test
    spike: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 10 },
        { duration: '1m', target: 50 },
        { duration: '30s', target: 100 },
        { duration: '1m', target: 100 },
        { duration: '30s', target: 0 },
      ],
      startTime: '6m',
    },
  },
  thresholds: {
    'git_clone_throughput_mbps': ['avg>10'],  // 10 MB/s target
    'git_clone_success': ['rate>0.99'],
    'http_req_failed': ['rate<0.01'],
  },
};

// Generate info/refs request
function infoRefsRequest(gitUrl) {
  return http.get(
    `${gitUrl}/info/refs?service=git-upload-pack`,
    {
      headers: {
        'Accept': 'application/x-git-upload-pack-advertisement',
      },
      timeout: '30s',
    }
  );
}

// Generate upload-pack request (fetch all)
function uploadPackRequest(gitUrl, wants) {
  // Build want lines
  let body = '';
  for (const oid of wants) {
    body += `want ${oid}\n`;
  }
  body += '\n'; // Flush packet
  body += 'done\n';

  return http.post(
    `${gitUrl}/git-upload-pack`,
    body,
    {
      headers: {
        'Content-Type': 'application/x-git-upload-pack-request',
        'Accept': 'application/x-git-upload-pack-result',
      },
      timeout: '60s',
      responseType: 'binary',
    }
  );
}

// Parse refs from info/refs response
function parseRefs(body) {
  const refs = [];
  const lines = body.split('\n');

  for (const line of lines) {
    // Skip pkt-line headers and capabilities
    if (line.includes('refs/')) {
      const match = line.match(/([0-9a-f]{40})\s+(\S+)/);
      if (match) {
        refs.push({ oid: match[1], name: match[2] });
      }
    }
  }

  return refs;
}

export function setup() {
  const baseUrl = __ENV.GUTS_URL || 'http://localhost:8080';
  const owner = __ENV.OWNER || 'loadtest';
  const repo = __ENV.REPO || 'test-clone-repo';

  // Create test repository with some content
  http.post(
    `${baseUrl}/api/repos`,
    JSON.stringify({ name: repo, owner: owner }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  return {
    baseUrl,
    owner,
    repo,
    gitUrl: `${baseUrl}/git/${owner}/${repo}`,
  };
}

export default function (data) {
  const startTotal = Date.now();

  // Step 1: Get refs (info/refs)
  const infoRes = infoRefsRequest(data.gitUrl);

  if (!check(infoRes, { 'info/refs OK': (r) => r.status === 200 })) {
    cloneErrors.add(1);
    cloneSuccess.add(false);
    return;
  }

  // Parse available refs
  const refs = parseRefs(infoRes.body);
  if (refs.length === 0) {
    // No refs to clone - repo is empty
    cloneSuccess.add(true);
    sleep(1);
    return;
  }

  // Step 2: Fetch objects (upload-pack)
  const wants = refs.map(r => r.oid);
  const packRes = uploadPackRequest(data.gitUrl, wants);

  const totalDuration = Date.now() - startTotal;
  cloneLatency.add(totalDuration);

  const success = packRes.status === 200;
  cloneSuccess.add(success);

  if (success && packRes.body) {
    const bodyLength = packRes.body.byteLength || packRes.body.length || 0;
    bytesReceived.add(bodyLength);

    // Calculate throughput in MB/s
    const durationSeconds = totalDuration / 1000;
    const throughputMbps = (bodyLength / (1024 * 1024)) / durationSeconds;
    cloneThroughput.add(throughputMbps);
  } else {
    cloneErrors.add(1);
  }

  check(packRes, {
    'upload-pack OK': (r) => r.status === 200,
  });

  // Think time between clones
  sleep(Math.random() * 3 + 2); // 2-5 seconds
}

// Continuous fetch simulation (incremental updates)
export function incrementalFetch(data) {
  const start = Date.now();

  // Just fetch refs to simulate checking for updates
  const infoRes = infoRefsRequest(data.gitUrl);

  check(infoRes, {
    'incremental fetch OK': (r) => r.status === 200,
  });

  const duration = Date.now() - start;
  cloneLatency.add(duration);

  // Short sleep for polling simulation
  sleep(5);
}

export function teardown(data) {
  console.log('Clone test completed');
}

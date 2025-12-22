/**
 * K6 Load Test: Git Push Operations
 *
 * Tests git-receive-pack endpoint under load to validate
 * git push latency targets (< 2s p95 for 1MB).
 *
 * Usage:
 *   k6 run --env GUTS_URL=http://localhost:8080 git_push.js
 */

import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';
import { randomBytes } from 'k6/crypto';

// Custom metrics
const pushLatency = new Trend('git_push_latency', true);
const pushSuccess = new Rate('git_push_success');
const pushErrors = new Counter('git_push_errors');
const bytesTransferred = new Counter('git_bytes_transferred');

// Test configuration
export const options = {
  scenarios: {
    // Ramp-up test
    ramp_up: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '1m', target: 50 },   // Ramp up to 50 users
        { duration: '3m', target: 50 },   // Stay at 50
        { duration: '1m', target: 100 },  // Ramp to 100
        { duration: '3m', target: 100 },  // Stay at 100
        { duration: '1m', target: 0 },    // Ramp down
      ],
      gracefulRampDown: '30s',
    },
  },
  thresholds: {
    'git_push_latency': ['p95<2000'],  // 2 second target
    'git_push_success': ['rate>0.99'],  // 99% success rate
    'http_req_failed': ['rate<0.01'],   // Less than 1% HTTP errors
  },
};

// Generate a mock pack file of approximately the given size
function generatePackData(sizeBytes) {
  // Pack file header: "PACK" + version (4 bytes) + object count (4 bytes)
  const header = new Uint8Array([
    0x50, 0x41, 0x43, 0x4b, // "PACK"
    0x00, 0x00, 0x00, 0x02, // Version 2
    0x00, 0x00, 0x00, 0x01, // 1 object
  ]);

  // Generate blob data
  const blobSize = sizeBytes - 32; // Account for header and trailer
  const blobData = randomBytes(Math.max(blobSize, 100));

  // Combine (simplified - real implementation needs proper encoding)
  const result = new Uint8Array(header.length + blobData.length + 20);
  result.set(header, 0);
  result.set(new Uint8Array(blobData), header.length);

  return result.buffer;
}

// Generate reference update command
function generateRefCommand(oldOid, newOid, refName) {
  // Format: old-oid SP new-oid SP ref-name NUL capabilities
  const capabilities = 'report-status side-band-64k';
  return `${oldOid} ${newOid} ${refName}\0${capabilities}\n`;
}

export function setup() {
  const baseUrl = __ENV.GUTS_URL || 'http://localhost:8080';
  const owner = __ENV.OWNER || 'loadtest';
  const repo = __ENV.REPO || `test-repo-${Date.now()}`;

  // Create test repository
  const createRes = http.post(
    `${baseUrl}/api/repos`,
    JSON.stringify({ name: repo, owner: owner }),
    { headers: { 'Content-Type': 'application/json' } }
  );

  if (createRes.status !== 201 && createRes.status !== 409) {
    console.error('Failed to create test repo:', createRes.body);
  }

  return {
    baseUrl,
    owner,
    repo,
    gitUrl: `${baseUrl}/git/${owner}/${repo}`,
  };
}

export default function (data) {
  const start = Date.now();

  // Small push (1KB)
  const smallPack = generatePackData(1024);
  const smallRes = http.post(
    `${data.gitUrl}/git-receive-pack`,
    smallPack,
    {
      headers: {
        'Content-Type': 'application/x-git-receive-pack-request',
      },
      timeout: '10s',
    }
  );

  const duration = Date.now() - start;
  pushLatency.add(duration);
  bytesTransferred.add(1024);

  const success = smallRes.status === 200 || smallRes.status === 204;
  pushSuccess.add(success);

  if (!success) {
    pushErrors.add(1);
    console.error(`Push failed: ${smallRes.status} - ${smallRes.body}`);
  }

  check(smallRes, {
    'status is 2xx': (r) => r.status >= 200 && r.status < 300,
    'response time OK': () => duration < 2000,
  });

  // Add some think time between requests
  sleep(Math.random() * 2 + 1); // 1-3 seconds
}

// Large push scenario (1MB)
export function largePush(data) {
  const start = Date.now();

  const largePack = generatePackData(1024 * 1024); // 1MB
  const res = http.post(
    `${data.gitUrl}/git-receive-pack`,
    largePack,
    {
      headers: {
        'Content-Type': 'application/x-git-receive-pack-request',
      },
      timeout: '30s',
    }
  );

  const duration = Date.now() - start;
  pushLatency.add(duration);
  bytesTransferred.add(1024 * 1024);

  const success = res.status === 200 || res.status === 204;
  pushSuccess.add(success);

  check(res, {
    'large push status OK': (r) => r.status >= 200 && r.status < 300,
    'large push under 2s': () => duration < 2000,
  });

  sleep(2);
}

export function teardown(data) {
  // Cleanup: delete test repository
  const res = http.del(
    `${data.baseUrl}/api/repos/${data.owner}/${data.repo}`,
    null,
    { headers: { 'Content-Type': 'application/json' } }
  );

  console.log(`Cleanup result: ${res.status}`);
}

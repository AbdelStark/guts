/**
 * K6 Load Test: API Read Operations
 *
 * Tests REST API read endpoints under load to validate
 * response time targets (< 100ms p99 for reads).
 *
 * Usage:
 *   k6 run --env GUTS_URL=http://localhost:8080 api_reads.js
 */

import http from 'k6/http';
import { check, sleep, group } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

// Custom metrics
const apiReadLatency = new Trend('api_read_latency', true);
const apiReadSuccess = new Rate('api_read_success');
const repoListLatency = new Trend('api_repo_list_latency', true);
const prListLatency = new Trend('api_pr_list_latency', true);
const issueListLatency = new Trend('api_issue_list_latency', true);

// Test configuration
export const options = {
  scenarios: {
    // Constant load
    constant_load: {
      executor: 'constant-vus',
      vus: 50,
      duration: '5m',
    },
    // Stress test
    stress: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '2m', target: 100 },
        { duration: '5m', target: 100 },
        { duration: '2m', target: 200 },
        { duration: '5m', target: 200 },
        { duration: '2m', target: 0 },
      ],
      startTime: '6m',
    },
  },
  thresholds: {
    'api_read_latency': ['p99<100'],     // 100ms target
    'api_read_success': ['rate>0.99'],
    'http_req_failed': ['rate<0.01'],
  },
};

export function setup() {
  const baseUrl = __ENV.GUTS_URL || 'http://localhost:8080';

  // Create test data
  const owner = 'loadtest';

  // Create multiple repositories
  const repos = [];
  for (let i = 0; i < 10; i++) {
    const repo = `test-repo-${i}`;
    http.post(
      `${baseUrl}/api/repos`,
      JSON.stringify({ name: repo, owner: owner }),
      { headers: { 'Content-Type': 'application/json' } }
    );
    repos.push({ owner, name: repo });
  }

  // Create some PRs and issues
  for (const repo of repos.slice(0, 3)) {
    for (let j = 0; j < 5; j++) {
      // Create PR
      http.post(
        `${baseUrl}/api/repos/${repo.owner}/${repo.name}/pulls`,
        JSON.stringify({
          title: `Test PR ${j}`,
          description: 'Load test PR',
          source_branch: `feature-${j}`,
          target_branch: 'main',
        }),
        { headers: { 'Content-Type': 'application/json' } }
      );

      // Create issue
      http.post(
        `${baseUrl}/api/repos/${repo.owner}/${repo.name}/issues`,
        JSON.stringify({
          title: `Test Issue ${j}`,
          description: 'Load test issue',
        }),
        { headers: { 'Content-Type': 'application/json' } }
      );
    }
  }

  return {
    baseUrl,
    repos,
  };
}

export default function (data) {
  const baseUrl = data.baseUrl;
  const repos = data.repos;

  // Pick a random repository
  const repo = repos[Math.floor(Math.random() * repos.length)];

  group('Repository Operations', () => {
    // List all repositories
    const start1 = Date.now();
    const listRes = http.get(`${baseUrl}/api/repos`, {
      headers: { 'Accept': 'application/json' },
      timeout: '10s',
    });
    const duration1 = Date.now() - start1;
    repoListLatency.add(duration1);
    apiReadLatency.add(duration1);
    apiReadSuccess.add(listRes.status === 200);

    check(listRes, {
      'list repos OK': (r) => r.status === 200,
      'list repos < 100ms': () => duration1 < 100,
    });

    // Get specific repository
    const start2 = Date.now();
    const getRes = http.get(
      `${baseUrl}/api/repos/${repo.owner}/${repo.name}`,
      {
        headers: { 'Accept': 'application/json' },
        timeout: '10s',
      }
    );
    const duration2 = Date.now() - start2;
    apiReadLatency.add(duration2);
    apiReadSuccess.add(getRes.status === 200 || getRes.status === 404);

    check(getRes, {
      'get repo OK': (r) => r.status === 200 || r.status === 404,
      'get repo < 100ms': () => duration2 < 100,
    });
  });

  group('Pull Request Operations', () => {
    // List pull requests
    const start = Date.now();
    const listRes = http.get(
      `${baseUrl}/api/repos/${repo.owner}/${repo.name}/pulls`,
      {
        headers: { 'Accept': 'application/json' },
        timeout: '10s',
      }
    );
    const duration = Date.now() - start;
    prListLatency.add(duration);
    apiReadLatency.add(duration);
    apiReadSuccess.add(listRes.status === 200);

    check(listRes, {
      'list PRs OK': (r) => r.status === 200,
      'list PRs < 100ms': () => duration < 100,
    });

    // Get specific PR if any exist
    if (listRes.status === 200) {
      try {
        const prs = JSON.parse(listRes.body);
        if (prs && prs.length > 0) {
          const pr = prs[0];
          const start2 = Date.now();
          const getRes = http.get(
            `${baseUrl}/api/repos/${repo.owner}/${repo.name}/pulls/${pr.number}`,
            {
              headers: { 'Accept': 'application/json' },
              timeout: '10s',
            }
          );
          const duration2 = Date.now() - start2;
          apiReadLatency.add(duration2);
          apiReadSuccess.add(getRes.status === 200);
        }
      } catch (e) {
        // Ignore parse errors
      }
    }
  });

  group('Issue Operations', () => {
    // List issues
    const start = Date.now();
    const listRes = http.get(
      `${baseUrl}/api/repos/${repo.owner}/${repo.name}/issues`,
      {
        headers: { 'Accept': 'application/json' },
        timeout: '10s',
      }
    );
    const duration = Date.now() - start;
    issueListLatency.add(duration);
    apiReadLatency.add(duration);
    apiReadSuccess.add(listRes.status === 200);

    check(listRes, {
      'list issues OK': (r) => r.status === 200,
      'list issues < 100ms': () => duration < 100,
    });
  });

  group('Organization Operations', () => {
    // List organizations
    const start = Date.now();
    const listRes = http.get(`${baseUrl}/api/orgs`, {
      headers: { 'Accept': 'application/json' },
      timeout: '10s',
    });
    const duration = Date.now() - start;
    apiReadLatency.add(duration);
    apiReadSuccess.add(listRes.status === 200);

    check(listRes, {
      'list orgs OK': (r) => r.status === 200,
      'list orgs < 100ms': () => duration < 100,
    });
  });

  // Think time
  sleep(Math.random() * 0.5 + 0.1); // 100-600ms
}

export function teardown(data) {
  console.log('API read test completed');
}

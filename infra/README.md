# Guts Infrastructure

This directory contains Infrastructure as Code (IaC) for deploying and running Guts nodes.

## Directory Structure

```
infra/
├── docker/                     # Docker configuration
│   ├── Dockerfile              # Multi-stage build for guts-node
│   ├── docker-compose.yml      # 3-node development cluster
│   └── docker-compose.devnet.yml  # 5-node test network
├── k8s/                        # Kubernetes manifests
│   ├── namespace.yaml          # Guts namespace
│   └── statefulset.yaml        # StatefulSet + Services
├── terraform/                  # AWS infrastructure
│   ├── main.tf                 # Main configuration
│   ├── modules/vpc/            # VPC module
│   └── templates/              # User data templates
└── scripts/                    # Helper scripts
    ├── devnet-start.sh         # Start 5-node devnet
    ├── devnet-stop.sh          # Stop devnet
    ├── devnet-status.sh        # Check devnet health
    └── devnet-e2e-test.sh      # Run E2E tests
```

## Quick Start: Local Devnet

Start a 5-node devnet for testing:

```bash
# Start the devnet
./scripts/devnet-start.sh --build --detach

# Check status
./scripts/devnet-status.sh

# Run E2E tests
./scripts/devnet-e2e-test.sh --skip-setup

# Stop the devnet
./scripts/devnet-stop.sh --volumes
```

### Node Endpoints

| Node | API Port | P2P Port |
|------|----------|----------|
| Node 1 | 8081 | 9001 |
| Node 2 | 8082 | 9002 |
| Node 3 | 8083 | 9003 |
| Node 4 | 8084 | 9004 |
| Node 5 | 8085 | 9005 |

## Docker

### Build Image

```bash
docker build -t guts-node -f docker/Dockerfile ../
```

### Run Single Node

```bash
docker run -p 8080:8080 -p 9000:9000 guts-node
```

### Development Cluster (3 nodes)

```bash
cd docker
docker compose up -d
```

### Test Network (5 nodes)

```bash
cd docker
docker compose -f docker-compose.devnet.yml up -d
```

## Kubernetes

Deploy to Kubernetes:

```bash
# Create namespace and deploy
kubectl apply -f k8s/

# Check status
kubectl get pods -n guts

# Access API
kubectl port-forward -n guts svc/guts-api 8080:80
```

### Components

- **Namespace**: `guts`
- **StatefulSet**: 3 replicas with persistent storage
- **Headless Service**: For internal P2P communication
- **LoadBalancer Service**: For external API access

## Terraform (AWS)

Deploy to AWS:

```bash
cd terraform

# Initialize
terraform init

# Plan
terraform plan -var="environment=dev" -var="node_count=5"

# Apply
terraform apply -var="environment=dev" -var="node_count=5"
```

### Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `aws_region` | us-east-1 | AWS region |
| `environment` | dev | Environment name |
| `node_count` | 3 | Number of nodes |
| `instance_type` | t3.medium | EC2 instance type |
| `docker_image` | ghcr.io/abdelstark/guts-node:latest | Docker image |

### Outputs

- `node_public_ips`: Public IPs of all nodes
- `node_private_ips`: Private IPs of all nodes
- `vpc_id`: VPC identifier

## E2E Testing

The devnet E2E test suite validates:

1. **Health Checks**: All 5 nodes respond
2. **Repository Creation**: Create repos across nodes
3. **Replication**: Verify data syncs between nodes
4. **Pull Requests**: Full PR workflow (create, comment, review, merge)
5. **Issues**: Create and comment on issues
6. **Organizations**: Create orgs, teams, manage members
7. **Consistency**: Cross-node data consistency
8. **Concurrency**: Parallel operations
9. **Webhooks**: Webhook configuration
10. **Branch Protection**: Protection rules
11. **Collaborators**: Access control

### Running Tests

```bash
# Full test (starts devnet, runs tests, stops devnet)
./scripts/devnet-e2e-test.sh

# Run against existing devnet
./scripts/devnet-e2e-test.sh --skip-setup

# Verbose output
./scripts/devnet-e2e-test.sh --verbose
```

## CI/CD Integration

The `devnet-e2e-extensive.yml` workflow:

1. Builds Docker image
2. Starts 5-node devnet
3. Runs E2E test suite
4. Runs load tests (50 repos, 100 issues, 200 concurrent reads)
5. Tests Git protocol endpoints
6. Collects logs on failure

Triggers:
- Pull requests to main
- Manual dispatch
- Nightly schedule (2 AM UTC)

## Troubleshooting

### Nodes not starting

Check Docker logs:
```bash
docker logs guts-devnet-node1
```

### Health check failures

Verify node is listening:
```bash
curl http://localhost:8081/health
```

### P2P connection issues

Check network connectivity:
```bash
docker network inspect guts-devnet
```

### Out of disk space

Remove unused volumes:
```bash
docker volume prune
```

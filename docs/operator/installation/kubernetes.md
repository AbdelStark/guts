# Kubernetes Deployment Guide

> Deploy Guts nodes on Kubernetes using Helm or raw manifests.

## Prerequisites

- Kubernetes 1.28+ cluster
- kubectl configured with cluster access
- Helm 3.12+ (for Helm deployment)
- StorageClass with dynamic provisioning
- LoadBalancer or Ingress controller (for external access)

## Helm Deployment (Recommended)

### Add Repository

```bash
helm repo add guts https://charts.guts.network
helm repo update
```

### Basic Installation

```bash
# Create namespace
kubectl create namespace guts

# Install with defaults
helm install guts-node guts/guts-node \
  --namespace guts

# Check status
kubectl get pods -n guts -w
```

### Production Installation

```bash
helm install guts-node guts/guts-node \
  --namespace guts \
  --create-namespace \
  --set replicaCount=3 \
  --set persistence.enabled=true \
  --set persistence.size=500Gi \
  --set persistence.storageClass=gp3 \
  --set resources.requests.cpu=2 \
  --set resources.requests.memory=8Gi \
  --set resources.limits.cpu=8 \
  --set resources.limits.memory=32Gi \
  --set metrics.enabled=true \
  --set serviceMonitor.enabled=true
```

### Custom Values File

Create `values.yaml`:

```yaml
# Replica configuration
replicaCount: 3

# Image settings
image:
  repository: ghcr.io/guts-network/guts-node
  tag: "latest"
  pullPolicy: IfNotPresent

# Service configuration
service:
  type: LoadBalancer
  api:
    port: 8080
  p2p:
    port: 9000
  metrics:
    port: 9090

# Persistence
persistence:
  enabled: true
  size: 500Gi
  storageClass: "gp3"
  accessModes:
    - ReadWriteOnce

# Resources
resources:
  requests:
    cpu: "2"
    memory: "8Gi"
  limits:
    cpu: "8"
    memory: "32Gi"

# Node configuration
config:
  logLevel: "info"
  logFormat: "json"
  consensus:
    enabled: true
    useSimplex: true
    blockTimeMs: 2000

# Probes
livenessProbe:
  httpGet:
    path: /health/live
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  httpGet:
    path: /health/ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3

# Monitoring
metrics:
  enabled: true
  serviceMonitor:
    enabled: true
    interval: 15s

# Security
podSecurityContext:
  runAsUser: 1000
  runAsGroup: 1000
  fsGroup: 1000

securityContext:
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: true
  runAsNonRoot: true
  capabilities:
    drop:
      - ALL

# Pod disruption budget
podDisruptionBudget:
  enabled: true
  minAvailable: 2

# Node affinity
affinity:
  podAntiAffinity:
    preferredDuringSchedulingIgnoredDuringExecution:
      - weight: 100
        podAffinityTerm:
          labelSelector:
            matchLabels:
              app.kubernetes.io/name: guts-node
          topologyKey: kubernetes.io/hostname
```

Install with custom values:

```bash
helm install guts-node guts/guts-node \
  --namespace guts \
  --create-namespace \
  -f values.yaml
```

## Raw Manifest Deployment

### Namespace

```yaml
# namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: guts
  labels:
    app.kubernetes.io/name: guts
```

### ConfigMap

```yaml
# configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: guts-config
  namespace: guts
data:
  config.yaml: |
    api:
      addr: "0.0.0.0:8080"
    p2p:
      addr: "0.0.0.0:9000"
    metrics:
      addr: "0.0.0.0:9090"
    logging:
      level: "info"
      format: "json"
    consensus:
      enabled: true
      use_simplex_bft: true
      block_time_ms: 2000
```

### Secret (Node Keys)

```yaml
# secret.yaml
apiVersion: v1
kind: Secret
metadata:
  name: guts-node-keys
  namespace: guts
type: Opaque
stringData:
  # Generate keys: guts-node keygen
  node-0.key: |
    <private-key-hex>
    <public-key-hex>
  node-1.key: |
    <private-key-hex>
    <public-key-hex>
  node-2.key: |
    <private-key-hex>
    <public-key-hex>
```

### StatefulSet

```yaml
# statefulset.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: guts-node
  namespace: guts
spec:
  serviceName: guts-node
  replicas: 3
  podManagementPolicy: Parallel
  selector:
    matchLabels:
      app.kubernetes.io/name: guts-node
  template:
    metadata:
      labels:
        app.kubernetes.io/name: guts-node
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "9090"
        prometheus.io/path: "/metrics"
    spec:
      securityContext:
        runAsUser: 1000
        runAsGroup: 1000
        fsGroup: 1000

      containers:
        - name: guts-node
          image: ghcr.io/guts-network/guts-node:latest
          imagePullPolicy: IfNotPresent

          ports:
            - name: api
              containerPort: 8080
              protocol: TCP
            - name: p2p-tcp
              containerPort: 9000
              protocol: TCP
            - name: p2p-udp
              containerPort: 9000
              protocol: UDP
            - name: metrics
              containerPort: 9090
              protocol: TCP

          env:
            - name: POD_NAME
              valueFrom:
                fieldRef:
                  fieldPath: metadata.name
            - name: GUTS_DATA_DIR
              value: /data
            - name: GUTS_LOG_LEVEL
              value: info
            - name: GUTS_LOG_FORMAT
              value: json

          volumeMounts:
            - name: data
              mountPath: /data
            - name: config
              mountPath: /etc/guts/config.yaml
              subPath: config.yaml
            - name: keys
              mountPath: /etc/guts/keys
              readOnly: true

          resources:
            requests:
              cpu: "2"
              memory: "8Gi"
            limits:
              cpu: "8"
              memory: "32Gi"

          livenessProbe:
            httpGet:
              path: /health/live
              port: 8080
            initialDelaySeconds: 30
            periodSeconds: 10
            timeoutSeconds: 5
            failureThreshold: 3

          readinessProbe:
            httpGet:
              path: /health/ready
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 5
            timeoutSeconds: 3
            failureThreshold: 3

          securityContext:
            allowPrivilegeEscalation: false
            readOnlyRootFilesystem: true
            runAsNonRoot: true
            capabilities:
              drop:
                - ALL

      volumes:
        - name: config
          configMap:
            name: guts-config
        - name: keys
          secret:
            secretName: guts-node-keys

  volumeClaimTemplates:
    - metadata:
        name: data
      spec:
        accessModes:
          - ReadWriteOnce
        storageClassName: gp3
        resources:
          requests:
            storage: 500Gi
```

### Services

```yaml
# services.yaml
---
# Headless service for P2P
apiVersion: v1
kind: Service
metadata:
  name: guts-node-headless
  namespace: guts
spec:
  clusterIP: None
  selector:
    app.kubernetes.io/name: guts-node
  ports:
    - name: p2p-tcp
      port: 9000
      targetPort: 9000
      protocol: TCP
    - name: p2p-udp
      port: 9000
      targetPort: 9000
      protocol: UDP

---
# LoadBalancer for API
apiVersion: v1
kind: Service
metadata:
  name: guts-node-api
  namespace: guts
  annotations:
    service.beta.kubernetes.io/aws-load-balancer-type: nlb
spec:
  type: LoadBalancer
  selector:
    app.kubernetes.io/name: guts-node
  ports:
    - name: http
      port: 80
      targetPort: 8080
      protocol: TCP
    - name: https
      port: 443
      targetPort: 8080
      protocol: TCP
```

### Apply Manifests

```bash
kubectl apply -f namespace.yaml
kubectl apply -f configmap.yaml
kubectl apply -f secret.yaml
kubectl apply -f statefulset.yaml
kubectl apply -f services.yaml
```

## Ingress Configuration

### NGINX Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: guts-ingress
  namespace: guts
  annotations:
    kubernetes.io/ingress.class: nginx
    nginx.ingress.kubernetes.io/proxy-body-size: "100m"
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
    - hosts:
        - guts.example.com
      secretName: guts-tls
  rules:
    - host: guts.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: guts-node-api
                port:
                  number: 80
```

## Monitoring Integration

### ServiceMonitor (Prometheus Operator)

```yaml
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: guts-node
  namespace: guts
  labels:
    release: prometheus
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: guts-node
  endpoints:
    - port: metrics
      interval: 15s
      path: /metrics
```

### PodMonitor (Alternative)

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PodMonitor
metadata:
  name: guts-node
  namespace: guts
spec:
  selector:
    matchLabels:
      app.kubernetes.io/name: guts-node
  podMetricsEndpoints:
    - port: metrics
      interval: 15s
```

## Network Policies

```yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: guts-node
  namespace: guts
spec:
  podSelector:
    matchLabels:
      app.kubernetes.io/name: guts-node
  policyTypes:
    - Ingress
    - Egress

  ingress:
    # Allow API from anywhere
    - ports:
        - port: 8080
          protocol: TCP

    # Allow P2P from anywhere
    - ports:
        - port: 9000
          protocol: TCP
        - port: 9000
          protocol: UDP

    # Allow metrics from monitoring namespace
    - from:
        - namespaceSelector:
            matchLabels:
              name: monitoring
      ports:
        - port: 9090
          protocol: TCP

  egress:
    # Allow all outbound
    - {}
```

## Pod Disruption Budget

```yaml
apiVersion: policy/v1
kind: PodDisruptionBudget
metadata:
  name: guts-node
  namespace: guts
spec:
  minAvailable: 2
  selector:
    matchLabels:
      app.kubernetes.io/name: guts-node
```

## Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: guts-node
  namespace: guts
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: StatefulSet
    name: guts-node
  minReplicas: 3
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80
```

## Backup with Velero

```yaml
# Schedule daily backups
apiVersion: velero.io/v1
kind: Schedule
metadata:
  name: guts-daily
  namespace: velero
spec:
  schedule: "0 2 * * *"
  template:
    includedNamespaces:
      - guts
    storageLocation: default
    volumeSnapshotLocations:
      - default
    ttl: 720h  # 30 days
```

## Troubleshooting

### Pod Not Starting

```bash
# Check pod status
kubectl describe pod -n guts guts-node-0

# Check logs
kubectl logs -n guts guts-node-0 --previous

# Check events
kubectl get events -n guts --sort-by='.lastTimestamp'
```

### Storage Issues

```bash
# Check PVC status
kubectl get pvc -n guts

# Check StorageClass
kubectl get storageclass

# Debug PVC
kubectl describe pvc -n guts data-guts-node-0
```

### Network Issues

```bash
# Test DNS resolution
kubectl exec -n guts guts-node-0 -- nslookup guts-node-headless

# Test connectivity
kubectl exec -n guts guts-node-0 -- curl -v http://guts-node-1.guts-node-headless:8080/health
```

## Next Steps

- [Configure networking](../configuration/networking.md)
- [Set up monitoring](../operations/monitoring.md)
- [Configure backups](../operations/backup.md)

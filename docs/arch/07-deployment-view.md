# 7. Deployment View

## 7.1 Kubernetes Deployment (Production)

```mermaid
C4Deployment
    title Deployment Diagram — Kubernetes (Production)

    Deployment_Node(k8s, "Kubernetes Cluster") {
        Deployment_Node(ingress_ns, "ingress-nginx namespace") {
            Container(nginx, "nginx Ingress", "nginx", "TLS termination, rate limiting, static asset caching")
        }
        Deployment_Node(app_ns, "req1 namespace") {
            Container(api1, "Axum API (replica 1)", "Rust", "REST API + SPA serving")
            Container(api2, "Axum API (replica 2)", "Rust", "REST API + SPA serving")
            Container(apiN, "Axum API (replica N)", "Rust", "Horizontally scaled")
        }
        Deployment_Node(data_ns, "data namespace") {
            ContainerDb(pg_primary, "PostgreSQL Primary", "PostgreSQL 16", "Read/write — data, history, baselines, FTS")
            ContainerDb(pg_replica, "PostgreSQL Replica", "PostgreSQL 16", "Read replicas for query scaling")
            ContainerDb(redis, "Redis", "Redis 7", "Sessions, cache, rate limiting")
        }
        Deployment_Node(storage_ns, "storage namespace") {
            ContainerDb(seaweed, "SeaweedFS", "SeaweedFS", "S3-compatible file storage with replication")
        }
    }

    Rel(nginx, api1, "Proxy", "HTTP")
    Rel(nginx, api2, "Proxy", "HTTP")
    Rel(nginx, apiN, "Proxy", "HTTP")
    Rel(api1, pg_primary, "SQL", "TCP")
    Rel(api2, pg_primary, "SQL", "TCP")
    Rel(api1, pg_replica, "Read queries", "TCP")
    Rel(api1, redis, "Sessions", "RESP")
    Rel(api2, redis, "Sessions", "RESP")
    Rel(api1, seaweed, "Files", "S3 API")
```

### Kubernetes Resource Summary

| Resource | Kind | Replicas | Notes |
|----------|------|----------|-------|
| nginx ingress | Ingress + IngressController | 2+ | TLS cert via cert-manager |
| Axum API | Deployment | 2+ (HPA) | Stateless, liveness + readiness probes |
| PostgreSQL Primary | StatefulSet | 1 | Persistent volume, WAL archiving |
| PostgreSQL Replica | StatefulSet | 1+ | Streaming replication, read-only |
| Redis | Deployment or StatefulSet | 1 | Persistent if needed for session durability |
| SeaweedFS | StatefulSet | 3+ | Master + volume servers, erasure coding |

### Infrastructure Configuration

| Aspect | Configuration |
|--------|--------------|
| TLS | cert-manager with Let's Encrypt or corporate CA |
| Secrets | Kubernetes Secrets or external vault (HashiCorp Vault) |
| Config | ConfigMaps + environment variables (12-factor) |
| Monitoring | Prometheus (scrape /metrics), Grafana dashboards |
| Tracing | OpenTelemetry Collector → Jaeger or Tempo |
| Logging | Structured JSON logs → Loki or ELK |
| Backups | pg_basebackup + WAL archiving to S3-compatible storage |

## 7.2 Docker Compose (Small Team / Development)

```yaml
# docker-compose.yml (simplified)
services:
  req1:
    image: req1:latest
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://req1:secret@postgres:5432/req1
      REDIS_URL: redis://redis:6379
      SEAWEEDFS_URL: http://seaweedfs:8333
      OIDC_ISSUER: https://idp.example.com
      OIDC_CLIENT_ID: req1
      OIDC_CLIENT_SECRET: ${OIDC_SECRET}
    depends_on:
      - postgres
      - redis
      - seaweedfs

  postgres:
    image: postgres:16
    volumes:
      - pgdata:/var/lib/postgresql/data
    environment:
      POSTGRES_DB: req1
      POSTGRES_USER: req1
      POSTGRES_PASSWORD: secret

  redis:
    image: redis:7-alpine
    volumes:
      - redisdata:/data

  seaweedfs:
    image: chrislusf/seaweedfs
    command: "server -s3"
    volumes:
      - seaweeddata:/data

volumes:
  pgdata:
  redisdata:
  seaweeddata:
```

### Single-Node Deployment Notes

- All services on one host — suitable for teams of 5–20 users
- nginx optional (Axum serves TLS directly or sits behind a host-level reverse proxy)
- Backup strategy: `pg_dump` cron job + volume snapshots
- Upgrade path: `docker compose pull && docker compose up -d`

## 7.3 Deployment Topology Comparison

| Aspect | Docker Compose | Kubernetes |
|--------|---------------|------------|
| Users | 5–20 | 20–1000+ |
| Scaling | Vertical only | Horizontal (HPA) |
| HA | No | Yes (multi-replica, PG failover) |
| Complexity | Low | Medium–High |
| Backup | pg_dump + cron | WAL archiving + PV snapshots |
| Monitoring | Docker logs | Prometheus + Grafana + OTel |
| TLS | Host reverse proxy or Axum | cert-manager + nginx ingress |

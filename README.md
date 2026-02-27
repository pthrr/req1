# req1

Open-source requirements management tool built to replace IBM DOORS Classic. Modern web UI for authoring, tracing, baselining, and reviewing requirements — powered by Rust, React, and PostgreSQL.

## Features

- **Hierarchical requirements** — Workspace > Project > Module > Object with outline numbering
- **Typed attributes** — string, int, float, bool, enum (single/multi), date, rich text, user reference
- **Directed traceability links** — named typed links (satisfies, derives-from, verifies, etc.) with suspect detection
- **Immutable baselines** — snapshots with word-level structured diffing
- **JavaScript scripting** — triggers (pre_save, pre_delete, validate), layout scripts, actions
- **Validation** — built-in structural rules + custom JavaScript rules
- **Full-text search** — PostgreSQL tsvector with GIN indexes
- **Impact analysis** — BFS graph traversal with D3 force-directed visualization
- **Coverage metrics** — upstream/downstream link coverage per module
- **HTML publishing** — Minijinja templates with configurable numbering
- **Saved views** — per-module column/filter/sort configurations
- **Object types** — schema-enforced typed objects with required attributes
- **Comments** — per-object discussion threads with resolve/unresolve
- **Soft delete** — recoverable object deletion
- **CLI** — headless automation via `req1-cli`

## Project Structure

```
req1/
├── .devcontainer/
│   ├── devcontainer.json     # VS Code devcontainer config
│   └── Dockerfile            # Ubuntu 24.04 + Rust + Bun + Docker CLI
├── crates/
│   ├── req1-server/          # Axum REST API server
│   │   ├── src/
│   │   │   ├── main.rs       # Server entrypoint (graceful shutdown, CORS, static serving)
│   │   │   ├── config.rs     # Environment config (PORT, CORS_ORIGIN, STATIC_DIR, BUILD_SHA)
│   │   │   ├── state.rs      # Shared application state
│   │   │   ├── error.rs      # Error types
│   │   │   ├── middleware.rs  # Cache-Control middleware for static assets
│   │   │   └── routes/       # 20 route modules (one per resource)
│   │   └── tests/
│   │       └── api_integration.rs
│   │
│   ├── req1-core/            # Business logic (no HTTP concerns)
│   │   └── src/
│   │       ├── service/      # 16 service modules (CRUD + business rules)
│   │       ├── scripting/    # JavaScript engine (V8 via deno_core) (triggers, layout, actions)
│   │       ├── validation.rs # Built-in validation rules
│   │       ├── baseline.rs   # Baseline snapshot + word-level diff
│   │       ├── fingerprint.rs # SHA-256 content fingerprinting
│   │       ├── suspect.rs    # Suspect link detection
│   │       ├── history.rs    # Object history recording
│   │       └── level.rs      # Outline level computation
│   │
│   ├── req1-cli/             # CLI client (talks to server via REST)
│   │   └── src/main.rs
│   │
│   └── req1-reqif/           # ReqIF import/export (stub)
│       └── src/lib.rs
│
├── entity/                   # Sea-ORM entities (21 database models)
├── migration/                # Sea-ORM migrations (23 sequential)
├── frontend/                 # React SPA (Vite, AG Grid, D3)
├── docs/                     # Architecture documentation (arc42)
├── Dockerfile                # Multi-stage production build (bun + cargo + debian-slim)
├── docker-compose.yml        # PostgreSQL, Redis, devcontainer service (dev)
├── docker-compose.prod.yml   # Production: app + postgres + redis
├── .dockerignore             # Docker build exclusions
├── Taskfile.yml              # Dev workflow tasks (task dev, task test, etc.)
├── flake.nix                 # Nix dev shell (alternative to devcontainer)
└── .env                      # Environment variables (DB, Redis, server config)
```

## Prerequisites

| Tool | Purpose | Install |
|------|---------|---------|
| Rust (stable) | Backend compilation | [rustup.rs](https://rustup.rs) |
| Bun | JS runtime, package manager | [bun.sh](https://bun.sh) |
| Docker | PostgreSQL + Redis containers | [docs.docker.com](https://docs.docker.com/get-docker/) |
| Task | Task runner | [taskfile.dev](https://taskfile.dev) |

Or use the Nix flake (`nix develop`) or the devcontainer (see below).

## Quick Start

```bash
# 1. Clone and enter the repository
git clone <repo-url> && cd req1

# 2. Enter a dev environment (pick one)
nix develop          # Option A: Nix flake
#                    # Option B: Open in devcontainer (see below)

# 3. Install frontend dependencies
task frontend:install

# 4. Start the full dev stack (DB + migrations + server + frontend)
task dev
```

This starts PostgreSQL + Redis containers, runs all migrations (fresh DB), builds the Rust workspace, starts the API server on `http://localhost:8080`, and opens the Vite dev server on `http://localhost:5173`.

### Devcontainer

If you can't or don't want to install dependencies natively, use the devcontainer instead. It ships with Rust, Bun, go-task, and Docker CLI pre-installed — no Nix required.

**VS Code:** Open the repo, run "Reopen in Container", then open a terminal.

**CLI:**

```bash
docker compose --profile dev run --rm dev bash
```

Then run:

```bash
task dev
```

The container uses `network_mode: host` and `pid: host`, and mounts the Docker socket, so `.env` and `task db` work unchanged. VS Code auto port forwarding is disabled (redundant with host networking).

## Development

### Task Commands

| Command | Description |
|---------|-------------|
| `task dev` | Full dev stack (DB + fresh migrate + build + server + frontend) |
| `task dev:stop` | Stop all dev processes and containers |
| `task build` | Build Rust workspace |
| `task check` | Type-check backend and frontend |
| `task clippy` | Lint Rust code |
| `task fmt` | Check formatting |
| `task fmt:fix` | Apply formatting |
| `task test` | Run all tests (backend + E2E) |
| `task test:backend` | Backend integration tests (requires running DB) |
| `task test:e2e` | Full E2E cycle (reset, build, start, Playwright, cleanup) |
| `task ci` | Full CI check (fmt + clippy + tsc + backend tests + E2E) |
| `task db` | Start PostgreSQL + Redis containers |
| `task db:reset` | Drop all tables and re-run migrations |
| `task db:migrate` | Run pending migrations only |

### Environment Variables

Configured in `.env`:

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `postgres://req1:req1dev@localhost:5432/req1` | PostgreSQL connection string |
| `REDIS_URL` | `redis://localhost:6379` | Redis connection string |
| `LISTEN_ADDR` | `0.0.0.0:8080` | API server listen address |
| `PORT` | — | Overrides `LISTEN_ADDR` with `0.0.0.0:{PORT}` (Cloud Run / Heroku compat) |
| `CORS_ORIGIN` | `*` (permissive) | Allowed origins, comma-separated. `*` or unset = permissive |
| `STATIC_DIR` | — | Path to frontend `dist/` directory for SPA serving |
| `BUILD_SHA` | — | Git commit SHA, included in `/health/live` and `/health/ready` responses |
| `RUST_LOG` | `req1_server=debug,tower_http=debug` | Log level filter |

### Testing

```bash
task test:backend         # 58 backend integration tests (sequential, shared DB)
task test:e2e             # 28 Playwright E2E tests (full lifecycle)
task ci                   # fmt + clippy + tsc + backend + E2E
```

### Adding a New Entity

1. Create migration in `migration/src/` (`m20260221_NNNNNN_description.rs`)
2. Register in `migration/src/lib.rs`
3. Add Sea-ORM entity in `entity/src/`, export in `entity/src/lib.rs`
4. Add service in `crates/req1-core/src/service/`
5. Add routes in `crates/req1-server/src/routes/`, register in `routes/mod.rs`

### Adding a New API Route

Routes follow a consistent CRUD pattern:

```rust
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/resources", get(list).post(create))
        .route("/resources/{id}", get(show).patch(update).delete(destroy))
}
```

### Database

PostgreSQL 16. Migrations are managed by Sea-ORM and run automatically on server startup. 21 tables across 23 migrations.

## Tech Stack

| Layer | Technology |
|-------|-----------|
| API server | Rust, Axum 0.8, Tower |
| ORM | Sea-ORM 1.x (PostgreSQL) |
| Scripting | deno_core (V8 JavaScript runtime) |
| Templates | Minijinja |
| Frontend | React 19, TypeScript, Vite 6 |
| Grid | AG Grid 33 |
| Graphs | D3 7 |
| E2E tests | Playwright |
| Database | PostgreSQL 16 |
| Cache | Redis 7 |
| Containers | Docker Compose |
| Dev shell | Nix flake / Devcontainer |
| Task runner | Task (go-task) |

## API Reference

Base URL: `http://localhost:8080`. All endpoints accept and return JSON. IDs are UUIDs (v7).

### Health

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health/live` | Liveness probe |
| GET | `/health/ready` | Readiness probe |

### Workspaces

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/workspaces` | List workspaces |
| POST | `/api/v1/workspaces` | Create workspace |
| GET | `/api/v1/workspaces/{id}` | Get workspace |
| PATCH | `/api/v1/workspaces/{id}` | Update workspace |
| DELETE | `/api/v1/workspaces/{id}` | Delete workspace |

### Projects

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/workspaces/{workspace_id}/projects` | List projects in workspace |
| POST | `/api/v1/workspaces/{workspace_id}/projects` | Create project |
| GET | `/api/v1/workspaces/{workspace_id}/projects/{id}` | Get project |
| PATCH | `/api/v1/workspaces/{workspace_id}/projects/{id}` | Update project |
| DELETE | `/api/v1/workspaces/{workspace_id}/projects/{id}` | Delete project |

### Modules

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/modules` | List modules |
| POST | `/api/v1/modules` | Create module |
| POST | `/api/v1/modules/from-template` | Create module from template |
| GET | `/api/v1/modules/{id}` | Get module |
| PATCH | `/api/v1/modules/{id}` | Update module |
| DELETE | `/api/v1/modules/{id}` | Delete module |

### Objects

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/modules/{module_id}/objects` | List objects (filtering, FTS, pagination) |
| POST | `/api/v1/modules/{module_id}/objects` | Create object |
| GET | `/api/v1/modules/{module_id}/objects/{id}` | Get object |
| PATCH | `/api/v1/modules/{module_id}/objects/{id}` | Update object |
| DELETE | `/api/v1/modules/{module_id}/objects/{id}` | Soft-delete object |
| GET | `/api/v1/modules/{module_id}/objects/{id}/history` | Object version history |

Object query parameters: `limit`, `offset`, `search`, `classification`, `needs_review`, `include_deleted`.

### Links

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/links` | List links |
| POST | `/api/v1/links` | Create link |
| GET | `/api/v1/links/{id}` | Get link |
| PATCH | `/api/v1/links/{id}` | Update link |
| DELETE | `/api/v1/links/{id}` | Delete link |
| GET | `/api/v1/link-types` | List link types |
| POST | `/api/v1/link-types` | Create link type |

### Baselines

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/modules/{module_id}/baselines` | List baselines |
| POST | `/api/v1/modules/{module_id}/baselines` | Create baseline (snapshot) |
| GET | `/api/v1/modules/{module_id}/baselines/{id}` | Get baseline |
| DELETE | `/api/v1/modules/{module_id}/baselines/{id}` | Delete baseline |
| GET | `/api/v1/modules/{module_id}/baseline-diff?a={id}&b={id}` | Diff two baselines (word-level) |

### Baseline Sets

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/baseline-sets` | List baseline sets |
| POST | `/api/v1/baseline-sets` | Create baseline set |
| GET | `/api/v1/baseline-sets/{id}` | Get baseline set |
| PATCH | `/api/v1/baseline-sets/{id}` | Update baseline set |
| DELETE | `/api/v1/baseline-sets/{id}` | Delete baseline set |

### Attribute Definitions

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/modules/{module_id}/attribute-definitions` | List attribute definitions |
| POST | `/api/v1/modules/{module_id}/attribute-definitions` | Create attribute definition |
| GET | `/api/v1/modules/{module_id}/attribute-definitions/{id}` | Get attribute definition |
| PATCH | `/api/v1/modules/{module_id}/attribute-definitions/{id}` | Update attribute definition |
| DELETE | `/api/v1/modules/{module_id}/attribute-definitions/{id}` | Delete attribute definition |

### Object Types

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/object-types` | List object types |
| POST | `/api/v1/object-types` | Create object type |
| GET | `/api/v1/object-types/{id}` | Get object type |
| PATCH | `/api/v1/object-types/{id}` | Update object type |
| DELETE | `/api/v1/object-types/{id}` | Delete object type |

### Scripts (JavaScript)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/modules/{module_id}/scripts` | List scripts |
| POST | `/api/v1/modules/{module_id}/scripts` | Create script |
| GET | `/api/v1/modules/{module_id}/scripts/{id}` | Get script |
| PATCH | `/api/v1/modules/{module_id}/scripts/{id}` | Update script |
| DELETE | `/api/v1/modules/{module_id}/scripts/{id}` | Delete script |
| POST | `/api/v1/modules/{module_id}/scripts/{id}/test` | Test script (dry run) |
| POST | `/api/v1/modules/{module_id}/scripts/{id}/execute` | Execute script |
| POST | `/api/v1/modules/{module_id}/scripts/{id}/layout` | Batch layout computation |

Script types: `trigger` (pre_save, pre_delete, validate), `layout` (computed columns), `action` (batch operations).

### Validation

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/modules/{module_id}/validate` | Validate module (built-in + JavaScript rules) |

Returns a report with issues (severity: error, warning, info), object count, and link count.

### Publishing

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/modules/{module_id}/publish?format=html` | Publish module to HTML |

### Views

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/modules/{module_id}/views` | List saved views |
| POST | `/api/v1/modules/{module_id}/views` | Create view |
| GET | `/api/v1/modules/{module_id}/views/{id}` | Get view |
| PATCH | `/api/v1/modules/{module_id}/views/{id}` | Update view |
| DELETE | `/api/v1/modules/{module_id}/views/{id}` | Delete view |

### Comments

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/objects/{object_id}/comments` | List comments on object |
| POST | `/api/v1/objects/{object_id}/comments` | Create comment |
| GET | `/api/v1/objects/{object_id}/comments/{id}` | Get comment |
| PATCH | `/api/v1/objects/{object_id}/comments/{id}` | Update / resolve comment |
| DELETE | `/api/v1/objects/{object_id}/comments/{id}` | Delete comment |

### Users

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/users` | List users |
| POST | `/api/v1/users` | Create user |
| GET | `/api/v1/users/{id}` | Get user |
| PATCH | `/api/v1/users/{id}` | Update user |
| DELETE | `/api/v1/users/{id}` | Delete user |

### Review Packages

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/modules/{module_id}/review-packages` | List review packages |
| POST | `/api/v1/modules/{module_id}/review-packages` | Create review package |
| GET | `/api/v1/modules/{module_id}/review-packages/{id}` | Get review package |
| PATCH | `/api/v1/modules/{module_id}/review-packages/{id}` | Update review package |
| DELETE | `/api/v1/modules/{module_id}/review-packages/{id}` | Delete review package |

### Review Assignments

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/review-packages/{package_id}/assignments` | List assignments |
| POST | `/api/v1/review-packages/{package_id}/assignments` | Create assignment |
| GET | `/api/v1/review-packages/{package_id}/assignments/{id}` | Get assignment |
| PATCH | `/api/v1/review-packages/{package_id}/assignments/{id}` | Update assignment |
| DELETE | `/api/v1/review-packages/{package_id}/assignments/{id}` | Delete assignment |

### Change Proposals

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/modules/{module_id}/change-proposals` | List change proposals |
| POST | `/api/v1/modules/{module_id}/change-proposals` | Create change proposal |
| GET | `/api/v1/modules/{module_id}/change-proposals/{id}` | Get change proposal |
| PATCH | `/api/v1/modules/{module_id}/change-proposals/{id}` | Update change proposal |
| DELETE | `/api/v1/modules/{module_id}/change-proposals/{id}` | Delete change proposal |

### Traceability

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/traceability-matrix?source_module_id={id}&target_module_id={id}` | Cross-module traceability matrix |
| GET | `/api/v1/modules/{module_id}/coverage` | Coverage metrics (upstream/downstream %) |

Optional query parameter: `link_type_id` to filter by link type.

### Impact Analysis

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/object-impact/{id}?direction={dir}&max_depth={n}` | BFS impact traversal |

Parameters: `direction` (forward, backward, both), `max_depth` (int).

## CLI

The `req1-cli` provides headless access to the server via REST.

```bash
cargo build -p req1-cli
# Binary: target/debug/req1
```

Configuration: `--url` flag or `REQ1_URL` env var (default: `http://localhost:8080`).

### List Resources

```bash
req1 list modules
req1 list modules --project-id <uuid>
req1 list objects --module-id <uuid>
req1 list objects --module-id <uuid> --tree
req1 list objects --module-id <uuid> --format json
req1 list links
req1 list links --module-id <uuid>
req1 list link-types
```

### Create Resources

```bash
req1 create object \
  --module-id <uuid> \
  --heading "Safety Requirement" \
  --body "The system shall..." \
  --classification normative \
  --parent-id <uuid>            # optional, for child objects

req1 create link \
  --source <uuid> --target <uuid> --link-type-id <uuid>

req1 create link-type \
  --name "derives-from" --description "Derived requirement link"
```

### Update Resources

```bash
req1 update object \
  --module-id <uuid> --object-id <uuid> \
  --heading "Updated Heading" \
  --classification informative
```

### Delete Resources

```bash
req1 delete object --module-id <uuid> --object-id <uuid>
req1 delete link --link-id <uuid>
```

### Validate

```bash
req1 validate --module-id <uuid>
```

Exits with code 1 on errors — suitable for CI pipelines.

### Review

```bash
req1 review --module-id <uuid>                        # review all unreviewed
req1 review --module-id <uuid> --object-id <uuid>     # review one object
```

### Resolve Suspect Links

```bash
req1 resolve-suspect --link-id <uuid>
```

### Publish

```bash
req1 publish --module-id <uuid> --format html --output module.html
```

### Output Formats

`list objects` supports three formats:

- **Table** (default) — columnar with level, ID, heading, classification, version, review status
- **Tree** (`--tree`) — indented hierarchy with `[R]`/`[ ]` review markers
- **JSON** (`--format json`) — full object data as pretty-printed JSON

## Production Deployment

### Docker (single container)

```bash
# Build the image (frontend + backend in one image)
docker build -t req1 .

# Or with a specific build SHA
docker build --build-arg BUILD_SHA=$(git rev-parse --short HEAD) -t req1 .
```

### Docker Compose (full stack)

```bash
# Start app + PostgreSQL + Redis
docker compose -f docker-compose.prod.yml up -d

# With custom Postgres password
POSTGRES_PASSWORD=mysecret docker compose -f docker-compose.prod.yml up -d

# Verify
curl http://localhost:8080/health/ready
```

The production Dockerfile uses a multi-stage build:
1. **Frontend** — `oven/bun:1` builds the React SPA via Vite
2. **Backend** — `rust:1-bookworm` compiles the Axum server in release mode
3. **Runtime** — `debian:bookworm-slim` with the binary + static assets, running as non-root (uid 1000)

The server serves the frontend as static files (SPA fallback to `index.html`) and applies `Cache-Control` headers (`immutable` for hashed `/assets/*`, `no-cache` for HTML).

## Documentation

Architecture docs (arc42): [`docs/`](docs/README.md)

## License

MIT

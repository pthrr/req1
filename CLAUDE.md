# req1

Requirements management tool — open-source replacement for IBM DOORS Classic.

## Quick Reference

```bash
task dev          # Full dev stack (DB + migrate + build + server + frontend)
task test         # All tests (backend + E2E)
task test:backend # Rust integration tests (requires running DB, --test-threads=1)
task test:e2e     # Playwright E2E (resets DB, builds, runs full lifecycle)
task clippy       # Lint Rust (-D warnings, strict)
task fmt:fix      # Auto-format Rust
task check        # Type-check backend + frontend
task db           # Start PostgreSQL + Redis containers
task db:reset     # Drop all tables + re-run migrations
```

## Architecture

```
crates/
  req1-server/   Axum REST API — routes, middleware, config, scheduler
  req1-core/     Business logic — services, scripting (deno_core/V8), validation
  req1-reqif/    ReqIF 1.2 XML library (serialize/deserialize, .reqifz archives)
  req1-cli/      CLI client over REST API (clap)
entity/          Sea-ORM entity models (36 models)
migration/       Sea-ORM migrations (41 migrations, auto-run on startup)
frontend/        React SPA (Vite + TypeScript + AG Grid + TipTap)
docs/            Arc42 architecture docs + SRS specification
```

### Request Flow

`Route handler (req1-server/routes/)` → `Service function (req1-core/service/)` → `Sea-ORM query (entity/)` → PostgreSQL

### Key Types

- `req1_core::Id` = `uuid::Uuid` — standard ID everywhere
- `req1_core::Pagination` — query param with offset/limit (default 0/50)
- `req1_core::PaginatedResponse<T>` — list response wrapper
- `AppError` (server) ← `CoreError` (core) — error types with `From` conversion

## Adding a New Feature

1. **Migration**: `sea-orm-cli migrate generate <name>` in project root
2. **Entity**: add model in `entity/src/<name>.rs`, register in `entity/src/lib.rs`
3. **Service**: add `crates/req1-core/src/service/<name>.rs`, register in `service/mod.rs`
4. **Route**: add `crates/req1-server/src/routes/<name>.rs`, register in `routes/mod.rs` and wire into `router()`
5. **Frontend**: add component in `frontend/src/components/`, page in `frontend/src/pages/`

## Conventions

- **Rust edition**: 2024, max line width 100 (`rustfmt.toml`)
- **Clippy**: strict `-D warnings` — includes `shadow_reuse`, `needless_pass_by_value`
- **Package manager**: **bun** (NOT npm) for frontend
- **Error handling**: `thiserror` enums, `CoreError` in core, `AppError` in server with `IntoResponse`
- **IDs**: UUID v7 via `uuid` crate
- **Async**: tokio runtime, all DB queries and handlers are async
- **Tests**: backend tests require running PostgreSQL (`task db`), run sequentially

## Scripting Engine (deno_core / V8)

- Engine: `crates/req1-core/src/scripting/engine.rs`
- Bootstrap JS: `crates/req1-core/src/scripting/bootstrap.js`
- `#[op2]` with `&mut OpState` — must `use deno_core::OpState`
- Error types need `#[derive(deno_error::JsError)]` with `#[class(generic)]`
- `#[op2(fast)]` incompatible with `#[serde]` params — use plain `#[op2]`
- `#[string] String` params need `#[allow(clippy::needless_pass_by_value)]`

## Environment

```bash
DATABASE_URL=postgres://req1:req1dev@localhost:5432/req1
REDIS_URL=redis://localhost:6379
LISTEN_ADDR=0.0.0.0:8080
RUST_LOG=req1_server=debug,tower_http=debug
```

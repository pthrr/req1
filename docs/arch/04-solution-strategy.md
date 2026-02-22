# 4. Solution Strategy

## 4.1 Technology Stack

> Items marked ✅ are implemented. Items marked 🔮 are planned for future phases.

| Layer | Choice | Status | Rationale |
|-------|--------|--------|-----------|
| Backend language | **Rust** (nightly, edition 2024) | ✅ | Performance, memory safety, single-binary deployment |
| Async runtime | **tokio** | ✅ | De facto standard for async Rust |
| Web framework | **Axum 0.8** | ✅ | Tokio-native, Tower middleware ecosystem, clean API ergonomics |
| ORM / DB access | **sea-orm 1.x** + **sqlx** | ✅ | sea-orm for ActiveRecord-style CRUD and migrations; drop to raw sqlx for recursive CTEs, dynamic filters, baseline diffs, graph traversal |
| Database | **PostgreSQL** | ✅ | Single source of truth. Relational integrity for links/attributes. JSONB for flexible attribute storage. Built-in FTS |
| Sessions / cache | **Redis** | 🔮 | Shared sessions across Axum replicas, cache for hot modules/views, rate limiting state |
| Object storage | **SeaweedFS** | 🔮 | S3-compatible API, self-hosted, lighter than MinIO. File attachments, images, embedded objects |
| Frontend | **Vite 6 + React 19 + TypeScript** | ✅ | AG Grid, D3, react-markdown — ecosystem for complex UI |
| Grid component | **AG Grid** (Community, v33) | ✅ | Enterprise-grade virtual-scrolling grid with inline editing, column configuration, filtering |
| Graph visualization | **D3** (force-directed) | ✅ | Impact analysis visualization with force-directed graph layout |
| Markdown rendering | **react-markdown** | ✅ | Renders requirement body content as markdown |
| Rich text editor | **TipTap** | 🔮 | Extensible ProseMirror-based editor for requirement body content |
| Tree view | **react-arborist** + **dnd-kit** | 🔮 | Hierarchical outline with drag-and-drop reordering |
| Routing | **React Router v7** | ✅ | Client-side routing with nested layouts |
| Authentication | **openidconnect** crate + **argon2** | 🔮 | OIDC for corporate SSO; argon2 for local password hashing (OWASP standard) |
| JWT | **jsonwebtoken** crate | 🔮 | API token generation and validation |
| ReqIF | **quick-xml** + **serde** (custom crate) | 🔮 | No existing Rust ReqIF crate. Structs matching ReqIF XSD with serde derive |
| HTML publishing | **minijinja** + **pulldown-cmark** | ✅ | Template-based HTML rendering with markdown-to-HTML conversion |
| PDF export | **typst** | 🔮 | Rust-native typesetting with programmable templates |
| DOCX export | **docx-rs** | 🔮 | Direct .docx generation from Rust |
| Content hashing | **sha2** | ✅ | SHA-256 fingerprinting for change detection and suspect link flags |
| CLI | **clap v4** (derive) | ✅ | Standard Rust CLI library for headless automation |
| Serialization | **serde** + serde_json | ✅ | Universal across the stack |
| Observability | **tracing** + tracing-subscriber | ✅ | Structured logging with env-filter and JSON output |
| Metrics / tracing | **opentelemetry** | 🔮 | Prometheus metrics, distributed tracing |
| Reverse proxy | **nginx** | 🔮 | TLS termination, static asset caching, rate limiting, load balancing |
| Deployment | **Docker Compose** (dev) / **Kubernetes** (prod) | ✅/🔮 | Dev: Docker Compose for PostgreSQL + devcontainer. Prod: horizontal scaling, health checks, rolling updates |
| Scripting engine | **mlua** (Lua 5.4) | ✅ | Embedded server-side scripting replacing DOORS Classic DXL. Sandboxed (memory/time limits). Triggers, layout scripts, actions. |
| OSLC | **Custom module** (JSON-LD + `serde`) | 🔮 | OSLC Core 3.0 service provider, OSLC-RM 2.1 resources. Delegated UI dialogs. Cross-tool traceability. |
| SysML v2 | **Custom module** (serde_json) | 🔮 | SysML v2 REST API requirements package. Import `RequirementUsage`/`RequirementDefinition`, export req1 modules. |
| MCP server | **Custom module** (JSON-RPC) | 🔮 | Model Context Protocol provider exposing req1 tools/resources to AI assistants. |
| E2E testing | **Playwright** | ✅ | Browser-based end-to-end testing (28 tests) |
| Backend testing | **tokio::test** + **reqwest** | ✅ | Integration tests against real PostgreSQL (58 tests) |
| Task runner | **Taskfile** (go-task) | ✅ | Dev workflow orchestration: build, test, ci, db:reset, dev |
| License | **MIT** | ✅ | Most permissive open-source license |

## 4.2 Key Architectural Approaches

### Database-First Versioning

PostgreSQL is the single source of truth for all versioning. Every mutation writes to a `object_history` table with attribute-level granularity. Baselines are sets of pointers to specific object versions. Baseline diffing is a SQL join — structured results, not text diffs.

### SPA + API

React single-page application communicates with the Rust backend via REST API. The frontend is compiled to static assets by Vite and served from the same Axum binary via `rust-embed` or `tower-http::ServeDir`. No Node.js process in production.

### Selective Server-Side Rendering

Not full SSR. Only for OpenGraph meta tags when chat clients (Slack/Teams) request shared URLs. Axum detects bot user-agents and serves minimal HTML with OG tags.

### Layered Database Access

sea-orm for CRUD scaffolding and migrations. Drop to raw sqlx for complex queries — recursive CTEs (object hierarchy), dynamic filter construction, baseline diffs, graph traversal.

### S3-Compatible Object Storage

SeaweedFS provides S3-compatible API for file attachments, images, and embedded objects. Self-hosted, no AWS dependency.

### Cross-Tool Traceability via OSLC

req1 acts as both OSLC provider (server) and consumer (client). As provider, it exposes requirement objects as OSLC-RM resources with a service provider catalog and delegated UI for selection/creation dialogs. As consumer, it can create links to resources hosted in any OSLC-capable tool (Polarion, Jama, DOORS Next, Jira with OSLC plugins). Links to external resources are stored alongside internal links with a URI reference instead of an internal object ID.

### Embedded Lua Scripting

Server-side Lua runtime (via `mlua`) provides DXL-equivalent scripting. Scripts access a safe subset of the data model (read/write objects, create links, query attributes) within a sandboxed environment with memory and CPU time limits. Scripts are stored as first-class entities, triggered via API or scheduled. This replaces the proprietary DXL language that locked DOORS Classic users into IBM's ecosystem.

### Delta-Aware Roundtrip Format

A custom JSON package format for export/reimport workflows. Each exported object includes content hashes and version metadata. After external editing (Excel, Word, another RM tool), reimport detects per-object deltas by comparing hashes. A 3-way merge UI presents added/modified/removed objects for accept/reject decisions.

## 4.3 Rejected Alternatives

### Storage Architecture

| Alternative | Reason for Rejection |
|-------------|---------------------|
| **Git-only (Doorstop model)** | Thousands of small YAML/TOML files cause `git status`/`git diff` slowdowns. Git has no semantic understanding of requirements — merge conflicts in structured data are not auto-resolvable. Blame is meaningless for attribute-level auditing. Works for 100–500 requirements but breaks at scale. |
| **Hybrid Git + DB** | Sync layer becomes non-trivial infrastructure. Every write updates two systems. Index rebuilds from Git add operational complexity. Risk of drift between two sources of truth. |
| **SQLite** | Concurrent write limitations. No JSONB. Harder to scale to multi-user server. Suitable for single-user/embedded only. |

### Web Framework

| Alternative | Reason for Rejection |
|-------------|---------------------|
| **actix-web** | Custom runtime on top of tokio (not tokio-native). Raw throughput advantage irrelevant for this workload. |
| **loco** | Built on Axum but too opinionated (Rails-like). Younger project, less control. |

### ORM

| Alternative | Reason for Rejection |
|-------------|---------------------|
| **diesel** | Primarily synchronous (`diesel-async` is beta). Heavier macro system. sea-orm + sqlx combo gives equivalent power with async-first design. |

### Frontend

| Alternative | Reason for Rejection |
|-------------|---------------------|
| **Next.js** | Optimized for public-facing apps (SSR, SEO, edge rendering) — not needed here. Adds Node.js server in production. App router / server component confusion. Vercel-oriented deployment model conflicts with self-hosted Axum. |
| **SvelteKit** | Grid ecosystem blocker — no official AG Grid bindings. Best native option (SVAR DataGrid) has ~129 stars. No tree component with DnD. Would require 2–4 weeks of custom work. |
| **Leptos (Rust WASM)** | No mature grid/table component. Would require building AG Grid equivalent from scratch (months). Tiny hiring pool. |
| **HTMX + Alpine.js** | Building a performant editable grid with virtual scrolling is fighting the tool. Fine for forms, not for spreadsheet-like interfaces. |
| **Tauri** | Same frontend question inside. Limits deployment to desktop only. Does not solve the grid problem. |

### Object Storage

| Alternative | Reason for Rejection |
|-------------|---------------------|
| **MinIO** | Heavier than SeaweedFS for the same workload. |
| **AWS S3** | Cloud dependency conflicts with self-hosted requirement. |

### Search

| Alternative | Reason for Rejection |
|-------------|---------------------|
| **Meilisearch** | Extra service to deploy. PostgreSQL FTS sufficient for MVP. Tantivy planned for later. |

### ReqIF

| Alternative | Reason for Rejection |
|-------------|---------------------|
| **Eclipse RMF (Java)** | Subprocess dependency on Java runtime. |

### Document Export

| Alternative | Reason for Rejection |
|-------------|---------------------|
| **Headless browser (weasyprint/chromium)** | External dependency. Rust-native solutions (typst, docx-rs) preferred. |

### Scripting Engine

| Alternative | Reason for Rejection |
|-------------|---------------------|
| **Embedded JavaScript (V8/deno_core)** | Large binary size (~20 MB V8). Complex build. Security surface area of a full JS engine. |
| **Embedded Python (PyO3)** | Requires Python runtime. Large dependency. GIL limits concurrency. |
| **WASM plugins** | Higher barrier for users writing scripts. Compilation step before execution. Fine for plugin authors, not for ad-hoc scripting by engineers. |
| **Rhai (Rust-native scripting)** | Rust-like syntax unfamiliar to most engineers. Smaller ecosystem than Lua. Less mature sandboxing. |

### Cross-Tool Protocol

| Alternative | Reason for Rejection |
|-------------|---------------------|
| **Custom REST linking API** | No standard discovery. Every tool integration is bespoke. OSLC is the industry standard for lifecycle tool interop. |
| **GraphQL federation** | Not a standard in the ALM/PLM ecosystem. OSLC is the established protocol for cross-tool traceability in regulated industries. |

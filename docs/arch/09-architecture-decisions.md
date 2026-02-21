# 9. Architecture Decisions

## ADR-1: PostgreSQL over Git for Data Storage

| | |
|---|---|
| **Status** | Accepted |
| **Context** | Requirements management tools need versioning, diffing, and branching. Git is a natural candidate. Doorstop uses Git (YAML files per requirement). We evaluated Git-only, hybrid Git+DB, and DB-only approaches. |
| **Decision** | Use PostgreSQL as the single source of truth. All versioning via `object_history` table with JSONB attribute snapshots. No Git dependency. |
| **Consequences** | (+) Structured diffs at attribute level, not text level. (+) Baselines are SQL queries, not branch snapshots. (+) No `git status` performance cliff at scale. (+) Relational integrity for links and attributes. (+) JSONB enables flexible attribute schemas without schema migrations. (-) Lose Git's built-in branching model — must implement branching in SQL. (-) Backup strategy must be explicit (pg_dump, WAL archiving). |

## ADR-2: sea-orm + sqlx for Database Access

| | |
|---|---|
| **Status** | Accepted |
| **Context** | Need an ORM for CRUD boilerplate and migrations, but also need raw SQL for complex queries (recursive CTEs, dynamic filters, graph traversal, baseline diffs). |
| **Decision** | Use **sea-orm** for ActiveRecord-style CRUD, migrations, and code generation. Drop to raw **sqlx** for complex queries. Both share the same connection pool since sea-orm is built on sqlx. |
| **Consequences** | (+) Best of both worlds: productivity for simple CRUD, power for complex queries. (+) Async-first design (unlike diesel). (+) Shared connection pool, no dual configuration. (-) Two query patterns in the codebase — contributors must know both. (-) sea-orm's entity code generation adds a build step. |

## ADR-3: Axum as Web Framework

| | |
|---|---|
| **Status** | Accepted |
| **Context** | Evaluated Axum, actix-web, and loco. Need tokio-native framework with Tower middleware compatibility for a long-lived project. |
| **Decision** | Use **Axum**. Tokio-native, Tower middleware ecosystem, clean extractor-based API. |
| **Consequences** | (+) Native tokio integration — no custom runtime layer. (+) Tower middleware reuse (tracing, CORS, compression, rate limiting). (+) Extractor pattern makes handler signatures self-documenting. (+) Strong community adoption. (-) Raw throughput slightly lower than actix-web (irrelevant for this workload). |

## ADR-4: React SPA over Next.js, SvelteKit, and Leptos

| | |
|---|---|
| **Status** | Accepted |
| **Context** | The primary UI is a spreadsheet-like grid for editing requirements. Need AG Grid (enterprise-grade virtual-scrolling grid), React Flow (graph visualization), TipTap (rich text), and react-arborist (tree with DnD). Evaluated React SPA, Next.js, SvelteKit, Leptos (Rust WASM), HTMX, and Tauri. |
| **Decision** | Use **Vite + React + TypeScript** as a single-page application. Static build served from the Axum binary. No Node.js in production. |
| **Consequences** | (+) Official AG Grid React bindings — battle-tested. (+) React Flow, TipTap, react-arborist, dnd-kit — all React-native. (+) Massive ecosystem, large hiring pool. (+) No SSR complexity — this is an authenticated internal app, not a public website. (-) Two-language stack (Rust + TypeScript) requires separate build pipelines. (-) Client-side rendering means no SEO (acceptable — authenticated app). |

### Why not the alternatives?

- **Next.js**: SSR/SEO/edge optimized for public web. Adds Node.js server. Vercel-oriented. Overkill for authenticated internal app.
- **SvelteKit**: No official AG Grid bindings. Best native grid (SVAR DataGrid, ~129 stars) is immature. No tree-with-DnD component. Would require 2–4 weeks custom work.
- **Leptos**: No mature grid component. Building AG Grid equivalent from scratch would take months. Tiny hiring pool.
- **HTMX + Alpine.js**: Cannot build a performant virtual-scrolling editable grid. Fine for forms, wrong for spreadsheets.
- **Tauri**: Same frontend framework question inside. Limits to desktop only.

## ADR-5: SeaweedFS for Object Storage

| | |
|---|---|
| **Status** | Accepted |
| **Context** | Need S3-compatible storage for file attachments, images, and embedded objects. Must be self-hostable. Evaluated AWS S3, MinIO, and SeaweedFS. |
| **Decision** | Use **SeaweedFS** with S3-compatible API. |
| **Consequences** | (+) Self-hosted, no AWS dependency. (+) Lighter than MinIO for the same workload. (+) Supports replication and erasure coding. (+) S3-compatible API means easy migration to AWS S3 or MinIO if needed. (-) Smaller community than MinIO. (-) Operational knowledge less common. |

## ADR-6: Redis for Sessions and Caching

| | |
|---|---|
| **Status** | Accepted |
| **Context** | Axum instances must be stateless for horizontal scaling. Need shared session storage, view caching, and rate limiting state. |
| **Decision** | Use **Redis** for shared sessions, cache, and rate limiting. |
| **Consequences** | (+) Enables stateless Axum replicas behind load balancer. (+) Sub-millisecond session lookups. (+) Built-in TTL for session expiry. (+) Rate limiting via sorted sets or token bucket. (-) Additional service to deploy and monitor. (-) Session loss on Redis restart (mitigated by persistent storage option). |

## ADR-7: Custom ReqIF Crate

| | |
|---|---|
| **Status** | Accepted |
| **Context** | ReqIF 1.2 is the OMG standard for requirements interchange. No Rust crate exists. Alternatives: wrap Eclipse RMF (Java subprocess), use Python library via subprocess, or build native Rust implementation. |
| **Decision** | Build a custom standalone Rust crate using **quick-xml** + **serde**. Define Rust structs matching the ReqIF XSD schema with serde derive for XML serialization. |
| **Consequences** | (+) No external runtime dependency (no JVM, no Python). (+) Type-safe ReqIF handling at compile time. (+) Can be published as a standalone crate for the Rust ecosystem. (+) Full control over import/export mapping. (-) Significant upfront effort to implement the full ReqIF schema. (-) Must validate against real-world DOORS Classic exports (edge cases). |

## ADR-8: typst for PDF Export

| | |
|---|---|
| **Status** | Accepted |
| **Context** | Need to generate PDF documents from requirements data (reports, traceability matrices, review packages). Evaluated headless browser (weasyprint/chromium), LaTeX, and typst. |
| **Decision** | Use **typst** for PDF generation. Supplement with **docx-rs** for Word output. |
| **Consequences** | (+) Rust-native — no external process, no browser, no LaTeX installation. (+) Programmable templates with typst scripting language. (+) Fast compilation. (+) Growing ecosystem and active development. (-) Younger than LaTeX — fewer templates and community resources. (-) Complex table layouts may require custom typst code. |

## ADR-9: OSLC Core 3.0 for Cross-Tool Traceability

| | |
|---|---|
| **Status** | Accepted |
| **Context** | Regulated industries (automotive, aerospace, medical) use multiple lifecycle tools. Traceability across tools is mandatory for compliance (ISO 26262, DO-178C). Options: (1) custom REST APIs per integration, (2) OSLC standard, (3) GraphQL federation. |
| **Decision** | Implement **OSLC Core 3.0** with **OSLC-RM 2.1** as both provider (server) and consumer (client). Use JSON-LD serialization. Provide delegated UI dialogs for selection and creation. Store external links alongside internal links. |
| **Consequences** | (+) Industry-standard protocol — compatible with Polarion, Jama, DOORS Next, Jira (with OSLC plugins), and any OSLC-compliant tool. (+) Delegated UI means no need to build custom UIs for every external tool. (+) Suspect detection works for external links too. (+) Service provider catalog enables automatic discovery. (-) OSLC spec is large (Core 3.0 + RM 2.1 + Query + Delegated Dialogs) — significant implementation effort. (-) JSON-LD/RDF serialization adds complexity vs. plain JSON. (-) OAuth 2.0 credential management for each external provider. |

## ADR-10: SysML v2 Requirements Package Only

| | |
|---|---|
| **Status** | Accepted |
| **Context** | SysML v2 is a complete modeling language with a REST/JSON API (OMG standard). Full SysML v2 support would require implementing the entire modeling stack. The user need is specifically to import/export requirements between req1 and MBSE tools. |
| **Decision** | Implement import/export for the **SysML v2 requirements package only** (`RequirementDefinition`, `RequirementUsage`, `SatisfyRequirementUsage`, `VerifyRequirementUsage`). Do not implement other SysML v2 packages (blocks, ports, flows, state machines). Optionally expose a lightweight SysML v2 API endpoint for the requirements subset. |
| **Consequences** | (+) Focused scope — months, not years. (+) Enables MBSE workflows without full modeling tool complexity. (+) JSON-based format is much simpler than SysML v1 XMI/XML. (+) SysML v2 API compatibility lets modeling tools query req1 directly. (-) SysML v2 standard is still evolving (OMG) — may need updates as the spec finalizes. (-) Traceability relationships in SysML v2 are richer than req1's link types — lossy mapping for non-requirements elements. |

## ADR-11: Lua over JavaScript/Python/WASM for Scripting

| | |
|---|---|
| **Status** | Accepted |
| **Context** | DOORS Classic users rely heavily on DXL (proprietary scripting) for automation. req1 needs an equivalent. Evaluated: (1) Lua via `mlua`, (2) JavaScript via V8/deno_core, (3) Python via PyO3, (4) WASM plugins, (5) Rhai (Rust-native scripting). |
| **Decision** | Embed **Lua 5.4** via the `mlua` crate. Expose a `req1.*` API surface to scripts. Sandbox with memory and CPU time limits. No file I/O, no network, no OS calls. |
| **Consequences** | (+) `mlua` is mature and well-maintained. (+) Lua is lightweight (~200 KB), embeds cleanly in Rust. (+) Familiar syntax — Lua is the most common embedded scripting language (game engines, Redis, nginx). (+) Built-in sandboxing: `mlua` safe mode disables `os`, `io`, `debug` libraries. (+) Fast startup — no JIT warmup needed. (-) Lua is less familiar to systems engineers than Python. (-) No native async — script execution blocks a thread (mitigated by running in a `spawn_blocking` task). (-) String manipulation is less ergonomic than Python. |

## ADR-12: Custom Roundtrip Format with Content Hashing

| | |
|---|---|
| **Status** | Accepted |
| **Context** | Users need to export requirements for external review/editing (Excel, Word) and reimport with delta detection. ReqIF supports round-trip but has no standard delta mechanism. Options: (1) extend ReqIF with custom attributes, (2) build a custom JSON-based format, (3) use CSV with hidden metadata. |
| **Decision** | Build a **custom JSON package format** (`.req1.json`) with per-object content hashes (SHA-256) and version metadata. Support Excel (`.req1.xlsx`) and CSV (`.req1.csv`) as alternative export targets with embedded hash metadata. Reimport compares hashes to detect deltas. 3-way merge UI for conflict resolution. |
| **Consequences** | (+) Precise per-object delta detection — no false positives from formatting changes. (+) 3-way merge enables safe concurrent editing across tools. (+) JSON package is programmatically accessible for script-based workflows. (+) Excel format enables business user workflows. (-) Custom format — not an industry standard. (-) Excel metadata (hidden sheet with hashes) can be accidentally deleted by users. (-) Must handle hash collisions (extremely unlikely with SHA-256 but documented). |

## ADR-13: MCP Server for AI Integration

| | |
|---|---|
| **Status** | Accepted |
| **Context** | AI assistants (Claude, etc.) can enhance requirements management through quality checks, coverage analysis, and natural language querying. Options: (1) custom chat API, (2) LangChain/LlamaIndex integration, (3) MCP (Model Context Protocol). |
| **Decision** | Implement an **MCP server** in req1 exposing tools and resources. Use streamable HTTP transport at `/api/mcp`. Authentication via existing session/JWT mechanism. |
| **Consequences** | (+) MCP is the standard protocol for AI tool integration — works with Claude and other MCP-compatible assistants. (+) Thin layer on top of existing REST API — low implementation effort. (+) Tools are self-describing — AI assistants discover capabilities automatically. (+) Resources enable contextual understanding of entire modules. (-) MCP is relatively new — protocol may evolve. (-) Quality check tools need domain-specific rules (INCOSE, EARS) that require upfront implementation effort. (-) AI-generated modifications must be clearly attributed in audit trail. |

## ADR-14: Risk Module as First-Class Entity

| | |
|---|---|
| **Status** | Accepted |
| **Context** | Risk analysis is mandatory in regulated industries (ISO 26262 HARA, ARP 4761, ISO 14971). Originally deferred to Phase 3 (TD7). Reconsidered because: hazard ↔ requirement traceability is a core need, not an add-on. Users cannot effectively demonstrate compliance without it. |
| **Decision** | Promote risk analysis from Phase 3 to **Phase 2**. Implement hazard register, risk assessments, and risk matrices as first-class entities. Link hazards to requirements via existing link system (`mitigates`, `addresses-hazard` link types). Support configurable severity/probability scales per safety standard. |
| **Consequences** | (+) Complete traceability chain: hazard → safety goal → requirement → verification. (+) Reuses existing link infrastructure — suspect detection applies. (+) Configurable scales support multiple standards (IEC 61508, ISO 26262, DO-178C, etc.). (+) Risk matrix and FMEA views differentiate req1 from simpler RM tools. (-) Adds entities, views, and API surface — increased MVP-to-Phase-2 scope. (-) Standard-specific scales need careful validation with domain experts. |

# 11. Risks and Technical Debt

## 11.1 Risks

| # | Risk | Probability | Impact | Mitigation |
|---|------|-------------|--------|------------|
| R1 | **No existing Rust ReqIF crate** — must build from scratch. ReqIF is well-specified (XSD) but verbose XML with many edge cases in real-world DOORS Classic exports. | High | High | Start with the XSD schema, define typed Rust structs with serde. Test against real DOORS Classic exports early. Publish as standalone crate for community validation. |
| R2 | **AG Grid community wrapper maturity** — using official React bindings (stable), but AG Grid Community Edition has fewer features than Enterprise. Some features users expect (row grouping, server-side row model) are Enterprise-only. | Medium | Medium | Evaluate AG Grid Community feature set against MVP requirements. Document which features require Enterprise license. Consider sponsoring or purchasing Enterprise license if needed. |
| R3 | **Enterprise OIDC edge cases** — corporate IdP configurations vary widely. Azure AD, Okta, Keycloak each have quirks (claim mapping, group sync, token lifetimes, PKCE support). | High | Medium | Test against multiple IdP providers early. Use the well-maintained `openidconnect` crate. Provide fallback local auth for environments without OIDC. Document known IdP-specific configurations. |
| R4 | **Performance at 100k+ objects unknown** — PostgreSQL + sea-orm performance for modules with 100,000+ objects is unproven for this workload. Grid virtual scrolling, FTS indexing, and baseline diffing may degrade. | Medium | High | Benchmark early with synthetic datasets (10k, 50k, 100k, 500k objects). Identify bottlenecks: pagination strategy, GIN index performance, JSONB query costs. Plan for read replicas and query optimization. |
| R5 | **Two-language stack complexity** — Rust backend + TypeScript frontend means two build pipelines, two dependency ecosystems, and an API contract that must stay synchronized. | Low | Medium | OpenAPI spec as the contract. Code-generate TypeScript API client from OpenAPI. CI validates API spec matches implementation. |
| R6 | **typst template complexity** — generating complex documents (traceability matrices, multi-column layouts, conditional formatting) may push typst's template language beyond comfortable limits. | Low | Medium | Prototype complex templates early. Typst is actively developed; engage with community. Fall back to direct PDF generation (printpdf) for edge cases. |
| R7 | **OSLC spec complexity** — OSLC Core 3.0 + RM 2.1 + Query + Delegated Dialogs is a large surface area. Real-world providers (Polarion, DOORS Next) have quirks and partial compliance. | High | Medium | Start with provider (server) role only — simpler, fewer external dependencies. Add consumer (client) role incrementally. Test against Lyo reference implementations. Validate with real enterprise toolchains early. |
| R8 | **SysML v2 standard not finalized** — OMG SysML v2 is still evolving. API specification and JSON schema may change. | Medium | Medium | Implement against the latest published draft. Abstract the mapping layer so schema changes only affect the serialization module, not core req1 data model. Pin to a specific SysML v2 API version. |
| R9 | **JavaScript sandbox escapes** — server-side scripting engines are a security surface. `deno_core` sandboxing restricts access but edge cases exist. | Low | High | Use `deno_core` sandboxing (no filesystem, no network, no OS access by default). Enforce memory and CPU limits. Run scripts in `spawn_blocking` with `tokio::time::timeout`. Security review of the `req1.*` API surface. Restrict `update_object` / `delete_object` to objects within user's RBAC scope. |
| R10 | **MCP protocol evolution** — MCP is a young protocol. Breaking changes or alternative standards may emerge. | Medium | Low | Implement as a thin adapter layer over the REST API. If MCP changes, only the adapter needs updating, not the core API. |

## 11.2 Technical Debt (Planned Shortcuts)

> Items marked ~~strikethrough~~ have been resolved.

| # | Debt | Reason for Deferral | Resolution Plan | Status |
|---|------|---------------------|-----------------|--------|
| TD1 | **PostgreSQL FTS instead of dedicated search engine** | PG FTS sufficient for MVP scale. Avoids deploying an additional service. | Phase 2: integrate tantivy for fuzzy matching, faceted search, and typo tolerance. | Open |
| TD2 | **No formal review workflow UI** | Review workflows require significant UI (participant management, per-object voting, status dashboards). | Phase 2: implement review UI with electronic signatures. | Partial — data model + CRUD routes done (`review_package`, `review_assignment`), UI pending |
| TD3 | **No AI-assisted quality checks** | Ambiguity detection, EARS pattern suggestions, and duplicate detection require NLP models and integration work. | Phase 2+: integrate LLM-based quality analysis (INCOSE rules, passive voice detection, vague term flagging). | Open |
| TD4 | **No variant management / branching** | DB-level branching with semantic merge is complex (branch contexts, merge resolution UI). | Phase 2: implement branching model in PostgreSQL with branch_id on object versions. | Open |
| TD5 | **No advanced reporting dashboard** | Coverage %, suspect link count, orphan detection, review status aggregation require dedicated views. | Phase 2: build project health dashboard with configurable widgets. | Partial — coverage endpoint + widget implemented |
| TD6 | **No custom report builder** | Users will want ad-hoc queries with exportable results. | Phase 2: attribute-based query builder with CSV/PDF/DOCX export. | Open |
| TD7 | **Risk module deferred from MVP** | Risk matrices, hazard register, FMEA views require dedicated entities and UI. Promoted from Phase 3 to Phase 2 (see ADR-14). | Phase 2: hazard register, risk assessments, risk matrices, integrity level assignment. | Open |
| TD8 | **No OSLC support** | OSLC Core 3.0 + RM 2.1 is a large standard. Cross-tool testing requires access to enterprise tools. | Phase 2: OSLC provider (server) first, then consumer (client). See ADR-9. | Open |
| TD9 | **No SysML v2 interchange** | SysML v2 standard is still evolving. JSON mapping needs domain validation. | Phase 2-3: requirements package import/export. See ADR-10. | Open |
| ~~TD10~~ | ~~**JavaScript scripting**~~ | ~~Scripting engine requires API surface design, sandboxing validation, and security review.~~ | ~~Phase 2: embedded JavaScript via deno_core.~~ | **Resolved** — trigger/layout/action scripts implemented with deno_core, sandboxed, batch layout endpoint, seed scripts |
| TD11 | **No MCP integration** | Thin layer over REST API but quality check rules (INCOSE, EARS) need implementation. | Phase 2-3: MCP server with tools and resources. See ADR-13. | Open |
| TD12 | **No roundtrip export format** | Custom format design, hash computation, and 3-way merge UI. | Phase 2: JSON/Excel/CSV roundtrip with delta detection. See ADR-12. | Open |
| TD13 | **No optional history policy** | Requires module-level config and conditional history insertion. | Phase 2: `history_policy` column on module with `always`/`on_baseline`/`off`. | Partial — module config exists (prefix, separator, digits, required_attributes, default_classification); history_policy field pending |
| TD14 | **No authentication** | No OIDC/JWT/session auth. All endpoints are unauthenticated. | Phase 2: OIDC + local auth, JWT tokens, RBAC enforcement. | Open |
| TD15 | **No import/export** | No CSV, XLSX, YAML, or ReqIF import/export. | Phase 1: CSV/YAML export/import. Phase 2: ReqIF. | Open |
| TD16 | **No lifecycle management** | No configurable state model on objects. PREEvision IREQ gap. | Phase 2: lifecycle model, states, transitions, color-coded display. | Open |
| TD17 | **No integrated test engineering** | No test case entity, no test execution tracking, no test coverage metrics. PREEvision IREQ gap. | Phase 2: test_case + test_execution entities, coverage dashboard. | Open |
| TD18 | **No reuse / placeholder mechanism** | Requirements cannot be embedded as synced references across modules. PREEvision IREQ gap. | Phase 2: placeholder objects with source sync + break-link. | Open |
| TD19 | **No Excel import/export** | Excel is the primary interchange format for stakeholder collaboration. PREEvision and most enterprise tools support it. | Phase 1-2: XLSX export/import with attribute column mapping. | Open |
| TD20 | **No review voting** | Review packages exist but no formal voting mechanism (approve/reject/abstain per reviewer). PREEvision IREQ gap. | Phase 2: voting entity + dashboard + review-scoped discussion. | Open |
| TD21 | **No deep links in published reports** | Published HTML has no hyperlinks back to the req1 model. PREEvision reports link back into the tool. | Phase 2: object-level deep links in published output. | Open |
| TD22 | **No rich text editor** | Requirement body editing is plain text / markdown in AG Grid cells. PREEvision has a full WYSIWYG editor with tables and graphics. | Phase 2: TipTap integration (already planned in solution strategy). | Open |
| TD23 | **No document view (LiveDoc)** | req1 is purely object/grid-based. Polarion's LiveDoc provides a Word-like document view where paragraphs are traceable objects. Regulated teams need document deliverables. | Phase 2-3: document view mode rendering module objects as continuous formatted document. | Open |
| TD24 | **No electronic signatures** | No re-authentication on workflow transitions. Required for FDA 21 CFR Part 11 (medical devices, pharma) and similar regulations. Polarion gap. | Phase 2-3: e-signature mechanism with audit records, four-eyes principle. | Open |
| TD25 | **No Word import/export** | No .docx round-trip. Polarion has rule-based Word import wizard and Word export. Many regulated processes require Word deliverables. | Phase 2: Word export via docx-rs (planned). Word import wizard. | Open |
| TD26 | **No compliance project templates** | No pre-configured templates for ISO 26262, DO-178C, IEC 62304. Polarion ships these out of the box. | Phase 2-3: template entities with attribute defs, object types, lifecycle models, validation rules. | Open |
| TD27 | **No script scheduling** | JavaScript scripts are on-demand only. Polarion supports CRON-based scheduled scripts. | Phase 2: CRON scheduling for JavaScript scripts, background job runner. | Open |
| TD28 | **No real-time collaboration** | No presence awareness or concurrent editing indicators. Polarion shows who else is editing. | Phase 2-3: WebSocket presence, editing indicators, save conflict warnings. | Open |
| TD29 | **No cross-project dashboards** | No configurable dashboard widgets. Polarion has LiveReport pages with drag-and-drop widgets. | Phase 2-3: dashboard entity, widget library, cross-module aggregation. | Open |
| TD30 | **No dependent enumerations** | Enum fields cannot cascade based on other enum selections. Polarion supports this. | Phase 2-3: dependent enum definitions on attribute_definition. | Open |
| TD31 | **No form layout designer** | Object editing uses a fixed form layout. Polarion allows per-type form configuration. | Phase 2-3: configurable form layout per object type. | Open |

## 11.3 Assumptions

| # | Assumption | If Wrong |
|---|-----------|----------|
| A1 | AG Grid Community Edition is sufficient for MVP grid features | Must evaluate Enterprise license cost and licensing compatibility with MIT |
| A2 | PostgreSQL JSONB performs well for attribute storage at scale | May need to normalize attributes into typed columns for hot paths |
| A3 | SeaweedFS is operationally stable for production file storage | Fall back to MinIO or local filesystem with S3 adapter |
| A4 | `openidconnect` crate handles all major IdP providers | May need custom provider-specific adapters |
| A5 | typst template language is expressive enough for requirement documents | Fall back to headless browser or LaTeX for complex layouts |
| A6 | `deno_core` sandboxing provides sufficient isolation for server-side scripts | May need additional isolation (separate process, seccomp, or WASM) |
| A7 | OSLC Core 3.0 JSON-LD serialization is supported by target tools (Polarion, DOORS Next) | May need to support RDF/XML or Turtle serialization as fallback |
| A8 | SysML v2 standard stabilizes before Phase 2 implementation | May need to track draft changes and update mapping layer |
| A9 | MCP protocol remains stable and gains broad adoption | May need to support alternative AI tool protocols |

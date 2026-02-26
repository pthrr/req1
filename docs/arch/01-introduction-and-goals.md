# 1. Introduction and Goals

## 1.1 Requirements Overview

**req1** is an open-source (MIT-licensed) requirements management tool built to replace IBM DOORS Classic. It provides a modern web-based interface for authoring, tracing, baselining, and reviewing requirements — capabilities that DOORS Classic delivers through a decades-old desktop client and proprietary scripting language (DXL).

### Core Problem

IBM DOORS Classic is end-of-life software with no modern alternative in the open-source space. Commercial tools like Vector PREEvision and Siemens Polarion ALM offer integrated requirements engineering with lifecycle management, test integration, review workflows, and document-based views — but as expensive proprietary applications (Eclipse RCP and Java/Jetty respectively) with per-seat licensing. Engineering teams in automotive, aerospace, medical devices, and defense need:

- Hierarchical requirements with typed attributes
- Directed traceability links with suspect detection
- Immutable baselines and structured diffing
- Configurable lifecycle management (state models with workflow transitions)
- Integrated test engineering (test cases linked to requirements, execution tracking)
- Document-based view (Word-like editing where paragraphs are traceable objects, à la Polarion LiveDoc)
- ReqIF, Excel, and Word interchange for tool interop and stakeholder collaboration
- Fine-grained access control for regulated environments
- Electronic signatures for compliance (FDA 21 CFR Part 11, ISO 26262)
- Review and approval workflows with formal voting
- Compliance project templates (ISO 26262, DO-178C, IEC 62304)
- A programmable API replacing DXL scripting, PREEvision's Java extensions, and Polarion's Velocity/Java SDK

### MVP Feature Set

| Feature | Description |
|---------|-------------|
| Modules | Hierarchical containers of requirements with outline numbering |
| Objects | Atomic traceable items with heading, body (rich text / markdown), and typed attributes |
| Attributes | Custom typed fields: string, integer, float, date, bool, enum (single/multi), rich text, user reference |
| Links | Directed, typed connections (satisfies, derives-from, verifies, mitigates, implements) with custom attributes on links |
| Suspect Detection | Auto-flag links when the source object is modified |
| Baselines | Immutable snapshots of module state with structured diff between baselines |
| Grid View | Spreadsheet-like primary editing interface with configurable columns |
| Filtering & Sorting | Attribute-based filters (equals, contains, date range, enum, regex) with saved views |
| ReqIF Import/Export | OMG ReqIF 1.2 interchange for DOORS Classic interoperability |
| REST API | OpenAPI-specified API covering all operations the UI can perform |
| RBAC | Role-based access control (admin, editor, reviewer, viewer) at project/module granularity |
| Full-Text Search | PostgreSQL FTS with tsvector/tsquery and GIN indexes |
| CLI | Command-line tool (clap v4) for headless automation and scripting |

### Phase 2 Feature Set

| Feature | Description |
|---------|-------------|
| Lifecycle Management | Configurable state model on objects (e.g., new → draft → in_review → approved → released). Color-coded status. Organization-specific workflow transitions. Inspired by PREEvision IREQ. |
| Test Engineering | Test case entity linked to requirements via `verifies`. Test execution tracking (status, executor, evidence). Test coverage metrics per module. |
| Excel Import/Export | XLSX import/export with column-to-attribute mapping, metadata preservation, and round-trip support. Primary interchange format for stakeholder collaboration. |
| Review Voting | Formal voting on requirements (approve / reject / abstain per reviewer). Voting dashboard with aggregated results. Review-scoped discussion threads. |
| OSLC Client & Server | OSLC Core 3.0 + OSLC-RM 2.1 provider and consumer. Cross-tool traceability via RDF/JSON-LD resources. Delegated selection/creation dialogs for linking to external tools. |
| SysML v2 Import/Export | Import `RequirementUsage` / `RequirementDefinition` from SysML v2 JSON. Export req1 modules as SysML v2 requirement elements. Enables MBSE workflows. |
| Lua Scripting Engine | Embedded server-side Lua (via `mlua`) replacing DOORS Classic DXL. Sandboxed scripts for batch operations, custom validation, reports, and workflow automation. |
| MCP Integration | Model Context Protocol server exposing tools (`search_requirements`, `get_object`, `create_link`, `run_coverage_analysis`, `check_requirement_quality`) to AI assistants. |
| Roundtrip Export Format | Delta-aware export format (JSON package with content hashes and version metadata). External edit, reimport, and 3-way merge with per-object accept/reject. |
| Risk Analysis Module | Hazard register, risk matrices (severity x probability), FMEA views. Link types: `mitigates`, `addresses-hazard`. Safety integrity level assignment (SIL/ASIL/DAL) per requirement. |
| Formal Review Workflow | Participant assignment, per-object approval/rejection with comments, review dashboards, electronic signatures. |
| Optional History Policy | Module-level toggle: `always` (full audit), `on-baseline` (snapshot on baseline only), `off` (no history). Reduces storage for large modules where full audit is not required. |
| Advanced Reporting | Coverage %, suspect link count, orphan detection, review status aggregation. Configurable dashboard widgets. Published reports with deep links back to model. |
| Custom Report Builder | Ad-hoc attribute-based query builder with CSV/PDF/DOCX export. |
| Reuse / Placeholders | Requirement placeholders that reference source objects in other modules. Sync mechanism keeps placeholders current. Break-link to create independent copies. |
| Document View (LiveDoc) | Word-like document editing mode where paragraphs are traceable objects. Dual nature: same data accessible in document view and grid/object view. Inspired by Polarion LiveDoc. |
| Electronic Signatures | Re-authentication on critical workflow transitions for FDA 21 CFR Part 11 compliance. Signature audit records. Four-eyes principle enforcement. |
| Word Import/Export | Rule-based Word import (map paragraphs to objects). Word export preserving document formatting. Round-trip support. |
| Compliance Templates | Pre-configured project templates for ISO 26262, DO-178C, IEC 62304, Automotive SPICE. Includes attribute definitions, object types, lifecycle models, and validation rules. |
| Script Scheduling | CRON-based scheduling for Lua scripts. Automated nightly validation, metric computation, stale link detection. |
| Real-Time Collaboration | WebSocket-based presence awareness. Concurrent editor indicators. Conflict warnings on simultaneous saves. |
| Cross-Project Dashboards | Dashboard pages with configurable widgets querying across modules and projects. Coverage charts, lifecycle distribution, test status, suspect link counts. |

## 1.2 Quality Goals

| Priority | Quality Goal | Description |
|----------|-------------|-------------|
| 1 | Usability | Modern web UI with spreadsheet-like grid as primary interface. Users work in it 8 hours/day. |
| 2 | Interoperability | ReqIF 1.2 round-trip without data loss. OSLC for cross-tool traceability. SysML v2 for MBSE integration. REST API for all operations. |
| 3 | Auditability | Every mutation recorded in a history table with attribute-level granularity. Immutable baselines. |
| 4 | Performance | Grid loads 10,000 objects in under 2 seconds. Responsive editing at scale. |
| 5 | Security | OIDC SSO, RBAC down to field level, CSP/HSTS headers, input sanitization. |

## 1.3 Stakeholders

| Role | Expectations |
|------|-------------|
| Systems Engineer | Author and trace requirements across modules. Edit in grid/outline views. Create baselines. Write Lua scripts for bulk operations. Manage requirement lifecycles. |
| Safety Engineer | Verify traceability coverage (ISO 26262, DO-178C, IEC 62304). Run impact analysis. Review baselines. Manage risk matrices and hazard traceability. |
| Test Engineer | Create and manage test cases linked to requirements. Track test execution and coverage. Verify traceability completeness. |
| Project Manager | Dashboard of project health — coverage %, suspect link count, review status, orphan count, risk status, lifecycle state distribution, test pass rates. |
| Admin | Configure OIDC providers, manage users/roles, set up projects and attribute schemas. Manage OSLC provider catalog. Define lifecycle models and workflow transitions. |
| Integrator / DevOps | Use REST API, CLI, and MCP tools for CI/CD pipelines and AI-assisted workflows. Configure webhooks. Deploy via containers. OSLC integration with external ALM/PLM tools. |
| Reviewer | Participate in formal reviews. Vote on requirements (approve/reject/abstain). Provide comments. Participate in review-scoped discussions. |
| MBSE Engineer | Import/export SysML v2 requirement elements. Maintain traceability between requirements and system models. |

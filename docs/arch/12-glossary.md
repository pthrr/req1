# 12. Glossary

## Domain Terms

| req1 Term | DOORS Classic Equivalent | Description |
|-----------|--------------------------|-------------|
| **Workspace** | Folder / Project hierarchy | Top-level organizational container. Isolates programs from each other. |
| **Project** | Project | Organizational unit within a workspace. Contains modules. |
| **Module** | Formal Module | Hierarchical container of requirements (objects). Represents a document or specification. |
| **Object** | Object | Atomic traceable item. Has a heading, body (rich text / markdown), and typed attributes. Every paragraph is uniquely identified and traceable. |
| **Attribute** | System / User Attribute | Typed metadata field on an object. Types: string, integer, float, date, bool, enum (single/multi), rich text, user reference. |
| **Attribute Definition** | Attribute Definition | Schema definition for an attribute: name, type, allowed values (for enums), default value. Can be module-scoped or global. |
| **Link** | Link (via Link Module) | Directed, typed connection between two objects (potentially across modules). Carries custom attributes (rationale, status, confidence). |
| **Link Type** | Link Module type | User-definable relationship type: satisfies, derives-from, verifies, mitigates, implements. |
| **Suspect Link** | Suspect Link | Link automatically flagged because the source object was modified after the link was created or last cleared. Indicates the link may need review. |
| **Baseline** | Module Baseline | Named, immutable snapshot of all objects in a module at a point in time. Stored as a set of `(object_id, version)` pointers. |
| **Baseline Set** | Global Configuration (DNG) | Cross-module collection of baselines representing a consistent system state. |
| **View** | View | Configured column layout with filters, sort order, and grouping. Can be personal or shared. |
| **Object Template** | Artifact Template (DNG) | Pre-defined object type with enforced attribute schema (e.g., stakeholder requirement, system requirement, test case, risk). |
| **Review** | N/A (DXL-based in Classic) | Formal review workflow. Participants are assigned, each reviews objects and submits decisions (approve / reject / defer). |
| **Branch** | N/A in Classic (DNG Streams) | Parallel editing context for variant engineering. Objects are versioned within a branch context. |
| **Hazard** | N/A | Risk analysis entity representing a potential danger. Linked to requirements via `mitigates` / `addresses-hazard` link types. |
| **Risk Assessment** | N/A | Evaluation of a hazard's severity and probability, producing a risk level. Multiple assessments per hazard (pre/post mitigation). |
| **Risk Matrix** | N/A | Visualization of hazards on a severity x probability grid with color-coded risk levels. |
| **Integrity Level** | N/A | Safety classification assigned to requirements: SIL (IEC 61508), ASIL (ISO 26262), DAL (DO-178C), Class (IEC 62304). |
| **Script** | DXL Script | Server-side JavaScript script for automation. Stored entity with sandboxed execution environment. |
| **External Link** | N/A | Link where the target is an external URI (OSLC resource) rather than an internal req1 object. |
| **Roundtrip Package** | N/A | Export format (.req1.json / .req1.xlsx) with content hashes enabling delta detection on reimport. |
| **Lifecycle** | N/A (PREEvision: Lebenszyklus) | Configurable state model defining the workflow states a requirement passes through (e.g., new → draft → in_review → approved → released). Color-coded in UI. |
| **Lifecycle Transition** | N/A | Allowed state change within a lifecycle model (e.g., draft → in_review). May require a specific role to execute. |
| **Test Case** | N/A (PREEvision: Testartefakt) | A test specification linked to requirements via `verifies` link type. Contains title, description, preconditions, and ordered steps with expected results. |
| **Test Execution** | N/A | Record of a test case run: executor, status (pass/fail/blocked), evidence, timestamp. |
| **Placeholder** | N/A (PREEvision: Platzhalter) | A proxy object in one module that references and stays in sync with a source object in another module. Read-only in the target module. |
| **Voting** | N/A (PREEvision: Abstimmung) | Formal decision mechanism on requirements: each reviewer submits approve / reject / abstain per artifact in a review package. |
| **Document View** | LiveDoc (Polarion) | Word-like rendering of a module's objects as a continuous formatted document. Each paragraph is a traceable object. Dual nature with grid view. |
| **E-Signature** | E-Signature (Polarion) | Re-authentication (username + password) on critical workflow transitions for regulatory compliance (FDA 21 CFR Part 11). Produces immutable audit record. |
| **Collection** | Collection (Polarion) | Cross-project grouping of document baselines that can be frozen together for compliance audits. Extends req1's baseline_set concept across projects. |
| **Compliance Template** | Project Template (Polarion) | Pre-configured project setup with attribute definitions, object types, lifecycle models, validation rules, and document templates for a specific standard (ISO 26262, DO-178C, etc.). |
| **Dependent Enumeration** | Dependent Enum (Polarion) | Enum field whose available values are filtered based on the selection in another enum field (cascading dropdown). |
| **Scheduled Script** | Scheduled Job (Polarion) | A JavaScript script with a CRON expression that executes automatically at defined intervals for proactive data quality and maintenance. |

## Technical Terms

| Term | Description |
|------|-------------|
| **PREEvision** | Vector Informatik's commercial E/E architecture and requirements management tool. Eclipse RCP-based. IREQ (Integrated Requirements Engineering) module provides lifecycle management, test integration, review voting, and ReqIF/Excel interchange. Key competitor for automotive RE. |
| **IREQ** | Integrated Requirements Engineering. PREEvision's approach where requirements interact with all development artifacts (architecture, test, wiring harness) in a shared model. |
| **Polarion ALM** | Siemens' commercial Application Lifecycle Management tool. Java/Jetty-based web application with SVN backend. Key innovation is LiveDoc (dual document + object model). Supports e-signatures, compliance templates, variant management (pure::variants), and extensive test management. Key competitor for regulated industries. |
| **LiveDoc** | Polarion's document concept where a Word-like document is composed of traceable Work Items (database objects). Every paragraph has metadata, workflow, and links while appearing as continuous narrative text. |
| **FDA 21 CFR Part 11** | US FDA regulation defining criteria for electronic records and electronic signatures. Requires re-authentication for signatures, audit trails, and record immutability. Relevant for medical device and pharmaceutical requirements management. |
| **ReqIF** | Requirements Interchange Format. OMG standard (v1.2) for exchanging requirements between tools. XML-based. |
| **OIDC** | OpenID Connect. Authentication protocol built on OAuth 2.0. Used for corporate SSO (Azure AD, Okta, Keycloak). |
| **RBAC** | Role-Based Access Control. Permissions assigned via roles (admin, editor, reviewer, viewer) scoped to workspaces, projects, or modules. |
| **PKCE** | Proof Key for Code Exchange. OAuth 2.0 extension preventing authorization code interception. Used in OIDC login flow. |
| **FTS** | Full-Text Search. PostgreSQL's built-in text search using `tsvector`, `tsquery`, and GIN indexes. |
| **JSONB** | PostgreSQL binary JSON type. Used for flexible attribute storage on objects — queryable, indexable, no schema migration needed for new attributes. |
| **CTE** | Common Table Expression. SQL `WITH` clause used for recursive queries (object hierarchy traversal) and complex multi-step queries. |
| **GIN Index** | Generalized Inverted Index. PostgreSQL index type used for full-text search and JSONB containment queries. |
| **OG Tags** | OpenGraph meta tags. HTML `<meta>` elements used by Slack/Teams to render link previews. |
| **ADR** | Architecture Decision Record. Structured documentation of a significant architectural decision (status, context, decision, consequences). |
| **DXL** | DOORS eXtension Language. Proprietary scripting language in DOORS Classic. req1's REST API and CLI replace DXL functionality. |
| **Tower** | Rust middleware framework used by Axum. Provides composable layers for logging, CORS, compression, rate limiting, etc. |
| **sea-orm** | Async ORM for Rust built on sqlx. Used for ActiveRecord-style CRUD and database migrations. |
| **sqlx** | Async SQL toolkit for Rust with compile-time query checking. Used for complex queries (CTEs, dynamic filters, baseline diffs). |
| **tantivy** | Rust-native full-text search engine (Lucene equivalent). Planned for Phase 2 to add fuzzy matching and faceted search. |
| **typst** | Rust-native typesetting system. Used for PDF document generation with programmable templates. |
| **SPA** | Single-Page Application. The React frontend runs entirely in the browser, communicating with the Axum backend via REST API. |
| **OSLC** | Open Services for Lifecycle Collaboration. OASIS standard for cross-tool integration in ALM/PLM toolchains. req1 implements Core 3.0 + RM 2.1. |
| **OSLC-RM** | OSLC Requirements Management domain specification (v2.1). Defines how requirements are exposed as linked data resources. |
| **JSON-LD** | JSON for Linked Data. W3C standard for expressing RDF graphs in JSON syntax. Used by OSLC Core 3.0 for resource serialization. |
| **RDF** | Resource Description Framework. W3C standard for data interchange on the web. Foundation of OSLC resource model. |
| **SysML v2** | Systems Modeling Language version 2 (OMG). Complete rewrite of SysML with REST/JSON API. req1 supports the requirements package subset. |
| **MBSE** | Model-Based Systems Engineering. Engineering approach using models as primary artifacts. SysML v2 integration enables MBSE workflows with req1. |
| **MCP** | Model Context Protocol. Standard protocol for AI assistants to interact with external tools and data sources via structured tool calls and resources. |
| **deno_core** | Rust crate providing V8 JavaScript engine bindings with built-in sandboxing. Used for the embedded scripting engine. |
| **JavaScript (V8)** | Scripting language executed via V8 engine. Used as DXL replacement for server-side automation scripts. |
| **FMEA** | Failure Mode and Effects Analysis. Systematic technique for identifying potential failure modes and their effects. Supported in the risk module. |
| **HARA** | Hazard Analysis and Risk Assessment. ISO 26262 method for identifying automotive hazards and assigning ASIL levels. |
| **Delegated Dialog** | OSLC UI pattern where an external tool opens a picker/creator dialog inside req1 (or vice versa) via iframe. |
| **Service Provider Catalog** | OSLC discovery mechanism. An endpoint listing available OSLC service providers and their capabilities. |

## Tools and Frameworks

| Term | Description |
|------|-------------|
| **Axum** | Async Rust web framework built on tokio and Tower. Serves the REST API and SPA static assets. |
| **tokio** | Async runtime for Rust. Provides event loop, I/O, and task scheduling for the backend. |
| **AG Grid** | Enterprise-grade JavaScript data grid with virtual scrolling, inline editing, and column configuration. Used as the primary editing interface (Community Edition). |
| **React Flow** | React library for building interactive node-based graphs. Used for traceability link visualization. |
| **TipTap** | Headless rich text editor framework built on ProseMirror. Used for editing requirement body content. |
| **react-arborist** | React tree view component. Used for hierarchical outline display of module objects. |
| **dnd-kit** | Drag-and-drop toolkit for React. Used for tree reordering in the outline view. |
| **Vite** | Frontend build tool. Bundles the React SPA into static assets for production. |
| **nginx** | Reverse proxy. Handles TLS termination, static asset caching, rate limiting, and load balancing. |
| **Redis** | In-memory data store. Used for shared sessions across Axum replicas, view caching, and rate limiting state. |
| **PostgreSQL** | Relational database. Single source of truth for requirements, links, history, baselines, and FTS indexes. |
| **SeaweedFS** | Distributed object storage with S3-compatible API. Stores file attachments, images, and embedded objects. |
| **quick-xml** | Rust crate for fast XML parsing and serialization. Used in the custom ReqIF crate. |
| **serde** | Rust serialization/deserialization framework. Used across the stack for JSON, YAML, and XML. |
| **docx-rs** | Rust crate for generating Word (.docx) documents directly. |
| **clap** | Rust CLI argument parser (v4, derive macros). Powers the req1 command-line tool. |
| **Docker** | Container runtime. All services packaged as OCI images. Docker Compose for dev services and devcontainer. |
| **Kubernetes** | Container orchestration platform. Production deployment target with horizontal scaling and health checks. |
| **deno_core** | Rust crate providing V8 JavaScript engine bindings. Powers the embedded scripting engine with sandboxing. |
| **sha2** | Rust crate for SHA-256 hashing. Used for content hashing in the roundtrip export format. |
| **Eclipse Lyo** | Java-based OSLC reference implementation. Used for interoperability testing of req1's OSLC provider/consumer. |
| **pure::variants** | Variant management tool by pure-systems, integrated into Polarion VARIANTS. Feature-model-based product line engineering for dynamically generating variant-specific artifact sets. |

## Compliance Standards

### Functional Safety and Integrity Standards

| Standard | Domain | Integrity Levels | Relevance to req1 |
|----------|--------|-----------------|---------------------|
| **IEC 61508** | General functional safety | SIL 1-4 | Foundation standard. Full traceability from hazard analysis through requirements to verification. Risk-based approach. |
| **ISO 26262** | Automotive functional safety | ASIL A-D (+ QM) | Requires HARA → safety goals → functional/technical safety requirements → verification traceability. ASIL decomposition. |
| **DO-178C** | Avionics software | DAL A-E | Requires bidirectional traceability (high-level req → low-level req → source code → test). Baseline integrity and data integrity critical. |
| **DO-254** | Avionics hardware (airborne electronics) | DAL A-E | Hardware requirements traceability. Complements DO-178C for complex electronic hardware. |
| **IEC 62304** | Medical device software | Class A-C | Requires requirements traceability, change management, and audit trail. Risk management per ISO 14971. |
| **ISO 14971** | Medical device risk management | — | Risk management process standard. Hazard analysis, risk estimation, risk evaluation, risk controls. Integrated with IEC 62304. |
| **EN 50128** | Railway software | SIL 0-4 | Software development lifecycle for railway control/protection systems. Requirements management and traceability. |
| **EN 50129** | Railway safety-related electronic systems | SIL 0-4 | System-level safety for railway. Safety case construction with requirements evidence. |
| **IEC 61511** | Process industry safety instrumented systems | SIL 1-3 | Safety instrumented functions for process plants. Hazard and risk analysis driving SIL assignment. |
| **ECSS-Q-ST-80C** | Space software | Criticality 1-4 | ESA standard for space software product assurance. Requirements engineering and verification. |
| **IEC 62443** | Industrial cybersecurity | SL 1-4 (Security Levels) | Security requirements for industrial automation and control systems. Security zones and conduits. |

### Engineering Practice Standards

| Standard | Domain | Relevance to req1 |
|----------|--------|--------------------|
| **INCOSE** | Systems engineering best practices | Quality rules for requirement writing (unambiguous, verifiable, atomic). AI checks via MCP tools. |
| **EARS** | Easy Approach to Requirements Syntax | Structured patterns for writing unambiguous requirements (ubiquitous, event-driven, state-driven, unwanted behavior, optional). AI suggestions via MCP. |
| **IEEE 29148** | Requirements engineering | Processes and products for requirements engineering. Requirement characteristics and quality criteria. |

### Export Control

| Standard | Domain | Relevance to req1 |
|----------|--------|--------------------|
| **ITAR** | US International Traffic in Arms Regulations | Data classification per module. Export restrictions enforced at API level. OSLC external links respect classification. |
| **EAR** | US Export Administration Regulations | Dual-use technology controls. Module-level classification determines export eligibility. |

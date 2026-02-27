# 8. Crosscutting Concepts

## 8.1 Authentication

### OIDC (Primary)

Corporate SSO via OpenID Connect using the `openidconnect` Rust crate. Supported providers: Azure AD, Okta, Keycloak, any OIDC-compliant IdP.

- Authorization Code Flow with PKCE
- MFA delegated to the IdP
- User provisioned on first login (JIT provisioning from ID token claims)
- Session stored in Redis, shared across Axum replicas
- CSRF protection via double-submit cookie pattern (Axum middleware)

### Local Auth (Fallback)

For deployments without a corporate IdP or for initial admin setup.

- Passwords hashed with **argon2** (OWASP recommended algorithm)
- JWT tokens issued via **jsonwebtoken** crate
- Password complexity and rotation policies configurable

### Session Management

- Session ID stored in HTTP-only, Secure, SameSite=Strict cookie
- Session data in Redis with configurable TTL
- Refresh token rotation for long-lived sessions
- Explicit logout invalidates session in Redis

## 8.2 Authorization (RBAC)

### Roles

| Role | Permissions |
|------|------------|
| **Admin** | Full access. User/role management. OIDC configuration. Project creation. |
| **Editor** | Create, read, update objects and links within assigned scope. Create baselines. |
| **Reviewer** | Read access. Submit review decisions (approve/reject). Add comments. |
| **Viewer** | Read-only access within assigned scope. |

### Permission Granularity

```
Workspace
  └── Project
        └── Module
              └── Object
                    └── Field (attribute-level)
```

- Roles are scoped to workspace, project, or module level
- Field-level permissions restrict visibility of specific attributes by role (e.g., classification fields visible only to cleared users)
- Program-level isolation: users in Program A cannot see Program B data
- Delegated administration: program leads manage access within their scope

### Module Locking

| Mode | Behavior |
|------|----------|
| **Exclusive edit** | Single user has write access; others read-only |
| **Shared edit** | Multiple editors; conflict resolution at save time |
| **Read-only** | No edits allowed (baselined or locked modules) |

### Export Control

- Data classification per module (e.g., ITAR, EAR)
- Export restrictions enforced at API level — classified data cannot be exported or accessed outside authorized scope

## 8.3 Audit Trail

Every mutation in the system is recorded with full attribution:

```sql
object_history (
    id              BIGSERIAL PRIMARY KEY,
    object_id       UUID NOT NULL,
    module_id       UUID NOT NULL,
    version         INTEGER NOT NULL,
    attribute_values JSONB NOT NULL,      -- full attribute snapshot
    changed_by      UUID NOT NULL,        -- user ID
    changed_at      TIMESTAMPTZ NOT NULL, -- server timestamp
    change_type     TEXT NOT NULL          -- 'create' | 'modify' | 'delete'
);
```

- **Attribute-level granularity**: JSONB snapshot enables diffing any two versions to see exactly which attributes changed
- **Immutable**: history rows are append-only, never updated or deleted
- **Queryable**: "who changed attribute X on object Y between dates A and B" is a simple SQL query
- **Compliance**: satisfies ISO 26262, DO-178C, IEC 62304 traceability requirements

## 8.4 Versioning and Baselines

### Object Versioning

- Every object mutation increments a version counter
- Full attribute snapshot stored in `object_history`
- Current state is always the latest version

### Baselines

- A baseline is a named, immutable set of pointers: `(object_id, version)` tuples
- Created atomically: one transaction snapshots all objects in a module
- Baseline entries reference `object_history` rows — data is never duplicated
- Baselines are locked after creation (no modifications)

### Baseline Diffing

Structured diff via SQL join:

```sql
SELECT a.object_id, a.version AS v_a, b.version AS v_b,
       ha.attribute_values, hb.attribute_values
FROM baseline_entry a
FULL OUTER JOIN baseline_entry b ON a.object_id = b.object_id
JOIN object_history ha ON ha.object_id = a.object_id AND ha.version = a.version
JOIN object_history hb ON hb.object_id = b.object_id AND hb.version = b.version
WHERE a.baseline_id = $1 AND b.baseline_id = $2
  AND (a.version != b.version OR a.object_id IS NULL OR b.object_id IS NULL);
```

Returns: added objects, removed objects, modified objects with per-attribute changes.

### Branching (Phase 2)

- DB-level branch contexts for variant engineering
- Semantic merge (attribute-level, not text-level)
- Cross-module baseline sets (DNG Global Configuration concept)

## 8.5 Search

### PostgreSQL Full-Text Search (MVP)

- `tsvector` columns on object body and attribute values
- GIN indexes for fast lookup
- `tsquery` with ranking (`ts_rank`) for relevance ordering
- Supports English, German, and other language configurations
- Phrase search, prefix matching, boolean operators

### Tantivy (Phase 2)

- Rust-native full-text search engine (Lucene equivalent)
- Typo tolerance (fuzzy matching)
- Faceted search (filter by attribute type, module, status)
- Near-real-time indexing from PostgreSQL change stream

## 8.6 ReqIF Interchange

### Implementation

- Custom standalone Rust crate using `quick-xml` + `serde`
- Rust structs matching the ReqIF 1.2 XSD schema
- Serde derive for XML serialization/deserialization

### Import Pipeline

1. Parse XML into typed Rust structs
2. Map `SPEC-TYPES` → req1 attribute definitions
3. Map `SPEC-OBJECTS` → req1 objects (with JSONB attributes)
4. Map `SPEC-RELATIONS` → req1 links
5. Bulk insert in a single transaction

### Export Pipeline

1. Query objects, attributes, links from PostgreSQL
2. Map to ReqIF struct hierarchy
3. Serialize to XML via `quick-xml`

### Round-Trip Fidelity

- All standard ReqIF attribute types preserved (string, integer, real, date, enum, XHTML)
- Custom attributes round-trip via `ATTRIBUTE-DEFINITION` mappings
- Embedded images/OLE objects stored in SeaweedFS, referenced in ReqIF output

## 8.7 Document Export

### PDF (typst)

- **typst** is a Rust-native typesetting system with programmable templates
- Templates define document structure: cover page, table of contents, requirement tables, traceability matrices
- Compiled directly in-process (no external subprocess)

### DOCX (docx-rs)

- **docx-rs** generates Word documents directly from Rust
- Template-based: heading styles, requirement tables, attribute columns
- Suitable for customers requiring editable Word deliverables

### Other Formats

| Format | Approach |
|--------|----------|
| CSV / Excel | Column mapping, configurable export |
| Markdown | Direct serialization of object body + attributes |
| HTML | Static site generation (StrictDoc-style) |

## 8.8 Observability

### Structured Logging

- `tracing` crate with `tracing-subscriber` for structured JSON output
- Log levels: ERROR, WARN, INFO, DEBUG, TRACE
- Contextual spans: request ID, user ID, module ID attached to every log entry

### Metrics

- OpenTelemetry metrics exported to Prometheus
- Key metrics: request latency (p50/p95/p99), active sessions, DB connection pool utilization, object count per module, import/export duration

### Distributed Tracing

- OpenTelemetry traces propagated across Axum → PostgreSQL → Redis → SeaweedFS
- Trace ID attached to every request for end-to-end debugging
- Export to Jaeger, Tempo, or any OTel-compatible backend

### Health Checks

- `/health/live` — process is running (liveness probe)
- `/health/ready` — DB, Redis, SeaweedFS connections verified (readiness probe)

## 8.9 Error Handling

### API Error Responses

- Consistent JSON error format: `{ "error": { "code": "...", "message": "...", "details": [...] } }`
- HTTP status codes follow REST conventions (400, 401, 403, 404, 409, 422, 500)
- Validation errors return field-level detail

### Security Headers

| Header | Value |
|--------|-------|
| Content-Security-Policy | Strict CSP preventing XSS |
| Strict-Transport-Security | `max-age=31536000; includeSubDomains` |
| X-Frame-Options | `DENY` |
| X-Content-Type-Options | `nosniff` |

### Input Sanitization

- All user input validated at the API boundary
- Rich text (XHTML) sanitized to prevent stored XSS
- SQL injection prevented by parameterized queries (sea-orm / sqlx)

## 8.10 OSLC (Open Services for Lifecycle Collaboration)

### Overview

req1 implements both OSLC provider (server) and consumer (client) roles, enabling cross-tool traceability in enterprise ALM/PLM toolchains.

### OSLC Provider (Server)

req1 exposes requirement objects as OSLC-RM 2.1 resources:

- **Service Provider Catalog** at `/.well-known/oslc/catalog` — advertises available OSLC services
- **Service Provider** per project — describes resource types, query capabilities, creation/selection dialogs
- **Requirement Resources** — each req1 object serialized as `oslc_rm:Requirement` in JSON-LD
- **Delegated UI Selection Dialog** — embedded picker for external tools to select req1 requirements
- **Delegated UI Creation Dialog** — embedded form for external tools to create req1 requirements
- **Query Capability** — OSLC query syntax (`oslc.where`, `oslc.select`, `oslc.prefix`) mapped to SQL

### OSLC Consumer (Client)

req1 can link to resources in external OSLC providers:

- **Provider Registration** — admin registers external OSLC service providers (catalog URI, OAuth credentials)
- **Discovery** — fetch service provider catalog, extract delegated selection dialog URIs
- **Selection** — open delegated dialog in iframe, receive selected resource URI via `postMessage`
- **External Links** — stored as links with `external_uri` field instead of `target_object_id`
- **Suspect Detection** — applies to external links when the source object is modified (same as internal links)

### Data Model Extension

```sql
-- External OSLC links stored alongside internal links
ALTER TABLE link ADD COLUMN external_uri TEXT;
-- NULL for internal links, non-NULL for OSLC cross-tool links

-- Registered external OSLC service providers
CREATE TABLE oslc_provider (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    catalog_uri TEXT NOT NULL,
    oauth_client_id TEXT,
    oauth_client_secret TEXT,  -- encrypted at rest
    created_at TIMESTAMPTZ DEFAULT now()
);
```

### Authentication with External Providers

- OAuth 2.0 client credentials flow for server-to-server OSLC queries
- OAuth 2.0 authorization code flow for delegated dialogs (user context)
- Credentials stored encrypted in PostgreSQL, decrypted at runtime

## 8.11 SysML v2 Interchange

### Scope

req1 imports and exports the **requirements package** of SysML v2 only — not the full modeling language. This covers:

- `RequirementDefinition` — maps to req1 attribute definitions / object templates
- `RequirementUsage` — maps to req1 objects
- `SatisfyRequirementUsage` — maps to req1 links (type: satisfies)
- `VerifyRequirementUsage` — maps to req1 links (type: verifies)
- `ConcernUsage` / `StakeholderUsage` — maps to req1 object metadata

### Import Pipeline

1. Parse SysML v2 JSON (REST API response or file)
2. Extract `RequirementUsage` elements → create req1 objects with heading/body/attributes
3. Extract `SatisfyRequirementUsage` / `VerifyRequirementUsage` → create req1 links
4. Preserve SysML v2 element IDs as external references for roundtrip fidelity
5. Bulk insert in a single transaction

### Export Pipeline

1. Query req1 module objects and links
2. Map objects → `RequirementUsage` with `declaredName`, `doc` (body), custom attributes as `AttributeUsage`
3. Map links → `SatisfyRequirementUsage` / `VerifyRequirementUsage`
4. Serialize as SysML v2 JSON
5. Optionally push to SysML v2 REST API endpoint

### SysML v2 API Compatibility

- Implements the SysML v2 REST API endpoint format for the requirements subset
- req1 can act as a lightweight SysML v2 API server for requirements only
- Full modeling tools (Cameo, Capella) can query req1 via standard SysML v2 API

## 8.12 JavaScript Scripting Engine

### Architecture

- **Runtime**: JavaScript (V8) via `deno_core` crate (sandboxed by default)
- **Sandboxing**: Per-execution limits — memory (configurable, default 64 MB), CPU time (configurable, default 30s), no file I/O, no network access, no OS calls
- **API Surface**: `req1.*` global object exposed to scripts

### Script API (`req1.*`)

| Function | Description |
|----------|-------------|
| `req1.find_objects(filter)` | Query objects by module, attribute filters, parent, FTS |
| `req1.get_object(id)` | Fetch single object with all attributes |
| `req1.update_object(id, changes)` | Update object attributes (triggers history + suspect) |
| `req1.create_object(module_id, data)` | Create new object |
| `req1.delete_object(id)` | Delete object |
| `req1.find_links(filter)` | Query links by source, target, type, suspect status |
| `req1.create_link(source, target, type)` | Create link |
| `req1.get_module(id)` | Fetch module metadata |
| `req1.find_baselines(module_id)` | List baselines for a module |
| `req1.log(message)` | Append to script execution log |
| `req1.context` | Read-only context: `user_id`, `module_id`, `project_id` |

### Storage and Execution

- Scripts stored in `script` table: `(id, name, body, description, created_by, created_at, updated_at)`
- Execution via API: `POST /api/scripts/{id}/execute` with context payload
- Inline execution: `POST /api/scripts/execute` with body + context (ad-hoc scripts)
- Execution results logged: `script_execution(id, script_id, user_id, started_at, duration_ms, status, result, objects_read, objects_written)`
- Audit trail: all data mutations made by scripts are attributed to the executing user

### Use Cases (DXL Equivalents)

| DOORS Classic DXL | req1 JavaScript |
|-------------------|----------|
| Batch attribute updates | `req1.find_objects` + `req1.update_object` in a loop |
| Custom validation rules | Script that checks each object against rules, returns violations |
| Coverage reports | `req1.find_links` + aggregate by type and status |
| Auto-numbering | `req1.find_objects` sorted by position, update heading prefix |
| Import from CSV | Parse CSV string, call `req1.create_object` per row |

## 8.13 MCP Integration

### Overview

req1 exposes a Model Context Protocol (MCP) server, enabling AI assistants (Claude, etc.) to interact with requirements data as structured tools.

### Transport

- **Streamable HTTP**: MCP endpoint at `/api/mcp` — stateless JSON-RPC over HTTP
- Authentication: same session/JWT as REST API — MCP requests carry user credentials
- Authorization: MCP tool calls are subject to the same RBAC as REST API calls

### Tools Exposed

| Tool | Description | Parameters |
|------|-------------|------------|
| `search_requirements` | Full-text search across modules | `query`, `module_id?`, `limit?` |
| `get_object` | Fetch requirement with full attributes | `object_id` |
| `get_module_objects` | List all objects in a module | `module_id`, `limit?`, `offset?` |
| `get_traceability` | Get links for an object (inbound + outbound) | `object_id` |
| `run_coverage_analysis` | Compute traceability coverage for a module | `module_id`, `link_type?` |
| `check_requirement_quality` | Analyze requirement text for quality issues | `object_id` or `text` |
| `diff_baselines` | Compare two baselines | `baseline_a`, `baseline_b` |
| `list_suspect_links` | Find all suspect links in a module | `module_id` |
| `create_link` | Create a traceability link | `source_id`, `target_id`, `link_type` |
| `update_object` | Modify requirement attributes | `object_id`, `changes` |

### Resources Exposed

| Resource | Description |
|----------|-------------|
| `req1://modules` | List of all accessible modules |
| `req1://modules/{id}/objects` | All objects in a module |
| `req1://modules/{id}/baselines` | All baselines for a module |
| `req1://objects/{id}/history` | Version history for an object |

### Quality Check Rules

The `check_requirement_quality` tool applies:

- **INCOSE rules**: ambiguity detection, passive voice, vague terms ("quickly", "easily", "appropriate"), incomplete conditions
- **EARS patterns**: checks if requirement follows EARS structured syntax (ubiquitous, event-driven, state-driven, unwanted behavior, optional)
- **Completeness**: missing required attributes, empty body, no traceability links
- **Duplicates**: fuzzy text similarity against other objects in the module

## 8.14 Roundtrip Export Format

### Package Structure

```json
{
  "format": "req1-roundtrip",
  "version": "1.0",
  "manifest": {
    "module_id": "uuid",
    "module_name": "System Requirements",
    "exported_at": "2026-03-15T10:30:00Z",
    "exported_by": "user-uuid",
    "object_count": 150,
    "hash_algorithm": "sha256"
  },
  "objects": [
    {
      "id": "object-uuid",
      "version": 3,
      "position": 0,
      "heading": "REQ-001",
      "body": "The system shall...",
      "attributes": { "status": "approved", "priority": "high" },
      "content_hash": "sha256:abc123..."
    }
  ]
}
```

### Hash Computation

Content hash is `SHA-256(canonical_json(heading + body + attributes))`. The canonical form ensures deterministic hashing regardless of JSON key ordering.

### Delta Detection on Reimport

For each object in the reimport package:

1. **Lookup** original export hash from the manifest
2. **Compute** hash of imported content
3. **Fetch** current DB content and compute its hash
4. **Classify**:
   - Hash matches original → **UNCHANGED** (skip)
   - Hash differs from original, original matches current DB → **MODIFIED externally** (clean merge)
   - Hash differs from original, original differs from current DB → **CONFLICT** (3-way merge needed)
   - Object not in original manifest → **ADDED** externally
   - Object in original but not in reimport → **REMOVED** externally

### Merge Resolution

- Clean modifications applied automatically (or with confirmation)
- Conflicts presented in a 3-way merge UI (original / external / current)
- User accepts/rejects per object or per field
- Applied changes create new object versions with `change_type = 'roundtrip_merge'`

### Supported Export Targets

| Format | Structure | Use Case |
|--------|-----------|----------|
| JSON (.req1.json) | Native package format | Programmatic editing, script-based transforms |
| Excel (.req1.xlsx) | Columns: ID, heading, body, attributes. Hidden metadata sheet with hashes. | Business user review and editing |
| CSV (.req1.csv) | Flat columns with hash column | Lightweight text-based editing |

## 8.15 Risk Analysis

### Data Model

```sql
hazard (
    id UUID PRIMARY KEY,
    project_id UUID NOT NULL REFERENCES project(id),
    name TEXT NOT NULL,
    description TEXT,
    source TEXT,                    -- e.g., 'HARA', 'FMEA', 'FTA'
    severity INTEGER NOT NULL,     -- 1-5 scale (configurable per standard)
    probability INTEGER NOT NULL,  -- 1-5 scale
    risk_level TEXT NOT NULL,      -- computed: 'low', 'medium', 'high', 'critical'
    integrity_level TEXT,          -- e.g., 'ASIL-D', 'SIL-3', 'DAL-A'
    status TEXT DEFAULT 'open',    -- open, mitigated, accepted, closed
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
);

risk_assessment (
    id UUID PRIMARY KEY,
    hazard_id UUID NOT NULL REFERENCES hazard(id),
    assessor_id UUID REFERENCES "user"(id),
    severity INTEGER NOT NULL,
    probability INTEGER NOT NULL,
    risk_level TEXT NOT NULL,
    rationale TEXT,
    assessed_at TIMESTAMPTZ DEFAULT now()
);
```

### Risk ↔ Requirement Traceability

Risks are linked to requirements via the existing `link` table:

- **Link type `mitigates`**: requirement → hazard (this requirement mitigates this hazard)
- **Link type `addresses-hazard`**: requirement → hazard (this requirement addresses this hazard)
- Suspect detection applies: if a mitigating requirement changes, the link is flagged suspect

### Integrity Level Assignment

- Objects can carry an `integrity_level` attribute (ASIL A-D, SIL 1-4, DAL A-E, etc.)
- Integrity level propagates through traceability: if a top-level requirement is ASIL-D, derived requirements inherit ASIL-D unless explicitly decomposed
- Coverage analysis: "Show all ASIL-D requirements without verified traceability links"

### Risk Matrix View

- Configurable grid: severity (columns) x probability (rows) → color-coded risk level
- Click cell to see hazards in that bucket
- Drill-down to linked requirements and their traceability status

### Supported Standards

| Standard | Severity Scale | Probability Scale | Risk Levels | Integrity Levels |
|----------|---------------|-------------------|-------------|-----------------|
| IEC 61508 | S1-S4 | W1-W3, Fr1-Fr3 | Tolerable, ALARP, Intolerable | SIL 1-4 |
| ISO 26262 | S0-S3 | E1-E4, C1-C3 | QM, ASIL A-D | ASIL A-D |
| DO-178C | Catastrophic → No Effect | Probable → Extremely Improbable | — | DAL A-E |
| IEC 62304 | — | — | — | Class A-C |
| EN 50128 | — | — | — | SIL 0-4 |
| IEC 61511 | — | — | — | SIL 1-3 |
| ECSS-Q-ST-80C | — | — | Criticality categories | Criticality 1-4 |
| IEC 62443 | — | — | — | SL 1-4 |

### FMEA View

Table-based Failure Mode and Effects Analysis:

| Column | Source |
|--------|--------|
| Component / Function | Object heading |
| Failure Mode | Hazard name |
| Severity | Hazard severity |
| Occurrence | Hazard probability |
| Detection | Risk assessment attribute |
| RPN | Computed (S × O × D) |
| Recommended Action | Linked mitigating requirement |

## 8.16 Lifecycle Management

### Motivation

Enterprise RE tools like PREEvision provide configurable lifecycle state models where requirements transition through defined states (e.g., new → draft → in_review → approved → released). Each state is color-coded for at-a-glance status visibility. req1 needs this to support organizational workflows and compliance processes.

### Data Model

```sql
lifecycle_model (
    id              UUID PRIMARY KEY,
    name            TEXT NOT NULL,          -- e.g., 'Standard Requirement Lifecycle'
    description     TEXT,
    created_at      TIMESTAMPTZ DEFAULT now()
);

lifecycle_state (
    id              UUID PRIMARY KEY,
    model_id        UUID NOT NULL REFERENCES lifecycle_model(id),
    name            TEXT NOT NULL,          -- e.g., 'draft', 'in_review', 'approved'
    color           TEXT NOT NULL,          -- hex color for UI display
    is_initial      BOOLEAN DEFAULT false,  -- starting state for new objects
    is_terminal     BOOLEAN DEFAULT false,  -- objects in this state are considered final
    sort_order      INTEGER NOT NULL
);

lifecycle_transition (
    id              UUID PRIMARY KEY,
    model_id        UUID NOT NULL REFERENCES lifecycle_model(id),
    from_state_id   UUID NOT NULL REFERENCES lifecycle_state(id),
    to_state_id     UUID NOT NULL REFERENCES lifecycle_state(id),
    name            TEXT,                   -- e.g., 'Submit for Review'
    required_role   TEXT,                   -- role required to perform this transition
    UNIQUE (model_id, from_state_id, to_state_id)
);
```

### Integration

- Modules reference a `lifecycle_model_id` — all objects in the module follow that lifecycle
- Object types can override the module default lifecycle
- Current lifecycle state stored on the object (FK to `lifecycle_state`)
- State transitions are audited in `object_history` with `change_type = 'state_transition'`
- Grid UI displays lifecycle state as a color-coded badge
- Transition buttons available in object detail and grid context menu

### Default Lifecycle

A built-in lifecycle model ships with req1:

| State | Color | Description |
|-------|-------|-------------|
| New | Blue | Object just created |
| Draft | Gray | Under active editing |
| In Review | Orange | Submitted for formal review |
| Approved | Green | Accepted by reviewers |
| Released | Dark Green | Baselined and released |
| Rejected | Red | Review rejected, needs rework |
| Deprecated | Light Gray | No longer active but retained for traceability |

## 8.17 Integrated Test Engineering

### Motivation

PREEvision and other enterprise RE tools integrate test management directly with requirements. Test cases linked to requirements via `verifies` links, test execution tracking, and test coverage metrics are essential for compliance (ISO 26262, DO-178C, IEC 62304).

### Data Model

```sql
test_case (
    id              UUID PRIMARY KEY,
    module_id       UUID NOT NULL REFERENCES module(id),
    title           TEXT NOT NULL,
    description     TEXT,
    preconditions   TEXT,
    steps           JSONB,                -- ordered list of { step, expected_result }
    object_type_id  UUID REFERENCES object_type(id),
    created_by      UUID REFERENCES app_user(id),
    created_at      TIMESTAMPTZ DEFAULT now(),
    updated_at      TIMESTAMPTZ DEFAULT now()
);

test_execution (
    id              UUID PRIMARY KEY,
    test_case_id    UUID NOT NULL REFERENCES test_case(id),
    executor_id     UUID REFERENCES app_user(id),
    status          TEXT NOT NULL,         -- 'pass', 'fail', 'blocked', 'not_run'
    evidence        TEXT,                  -- notes, log references
    executed_at     TIMESTAMPTZ DEFAULT now()
);
```

### Integration

- Test cases are linked to requirements via existing `link` table with link type `verifies`
- Test coverage metric: % of requirements with at least one linked test case with passing execution
- Test status dashboard: pass/fail/blocked/not_run breakdown per module
- Test execution history: track results over time for regression analysis

## 8.18 Reuse and Placeholders

### Motivation

PREEvision supports reusing requirements across modules via placeholders — embedded references that stay in sync with the source object. This avoids duplication while maintaining traceability.

### Mechanism

- A **placeholder** is a special object in the target module that references a source object in another module
- Placeholder displays the source object's current heading, body, and attributes (read-only)
- Changes to the source object are automatically reflected in all placeholders
- Placeholders participate in the target module's link structure and baselines
- **Break link** action converts a placeholder into an independent copy (severs the sync)
- Placeholder objects are visually distinguished in the grid (icon + read-only styling)

### Data Model Extension

```sql
-- On object table
ALTER TABLE object ADD COLUMN placeholder_source_id UUID REFERENCES object(id);
-- NULL for native objects, non-NULL for placeholders
```

## 8.19 Document View (LiveDoc-Style)

### Motivation

Polarion ALM's core innovation is the LiveDoc: a Word-like document where every paragraph is simultaneously a traceable, workflow-controlled database object. Many regulated industries require document deliverables (SRS, SDD, SVP) while engineering teams need object-based traceability. A document view bridges both worlds.

### Approach

- A **document view** renders a module's objects as a continuous, formatted document (like Microsoft Word or Google Docs)
- Each paragraph/section in the document corresponds to an object in the module
- Users can switch between **document view** (narrative reading/authoring) and **grid view** (structured filtering/editing)
- Edits in either view are reflected in the other — same underlying data model
- Document view supports rich text formatting, embedded images, and tables via TipTap

### Document Outline

- Sidebar navigator showing document heading hierarchy
- Click-to-scroll navigation within large documents
- Drag-and-drop reorder of sections (updates object position and parent)

### Export

- Document view exports to Word (.docx) preserving formatting and styles
- Document view exports to PDF via typst templates
- Exported documents include deep links back to the req1 model

## 8.20 Electronic Signatures

### Motivation

Regulated industries (medical devices, pharma, avionics) require electronic signatures on critical workflow transitions for compliance with FDA 21 CFR Part 11, EU Annex 11, and similar regulations. This is more rigorous than review voting — it requires re-authentication.

### Mechanism

- Certain workflow transitions can be configured to **require an e-signature**
- When triggered, the user must re-enter their credentials (username + password) in a confirmation dialog
- The signature is recorded as an immutable audit record

### Data Model

```sql
e_signature (
    id              UUID PRIMARY KEY,
    object_id       UUID NOT NULL REFERENCES object(id),
    signer_id       UUID NOT NULL REFERENCES app_user(id),
    transition_name TEXT NOT NULL,          -- e.g., 'Approve', 'Release'
    meaning         TEXT NOT NULL,          -- e.g., 'I approve this requirement'
    signature_hash  TEXT NOT NULL,          -- hash of (user_id + object_id + version + timestamp + meaning)
    signed_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

### Compliance Features

- **Four-eyes principle**: configurable enforcement that signer ≠ author
- **Signature meaning**: each transition defines a meaning statement the signer confirms
- **Immutable records**: signature rows are append-only, never updated or deleted
- **Audit queryable**: "who signed what, when, and with what meaning" is a simple SQL query

## 8.21 Script Scheduling

### Motivation

Polarion supports CRON-based server-side script execution for automated data quality. req1's JavaScript scripting engine currently supports only on-demand execution (API trigger). Scheduled scripts enable proactive maintenance.

### Implementation

- Scripts gain an optional `cron_expression` field (e.g., `0 2 * * *` for nightly at 2 AM)
- A background job runner (tokio task) evaluates CRON expressions and triggers script execution
- Execution results logged in `script_execution` table with status, duration, output
- Failed executions generate notifications (webhook or email)

### Use Cases

- Nightly validation of all modules (run built-in + custom JavaScript rules)
- Stale suspect link detection (flag links that have been suspect for > N days)
- Coverage metric computation and caching
- Automated report generation

## 8.22 Real-Time Collaboration

### Motivation

Polarion shows visual indicators when multiple users edit the same document simultaneously. For a multi-user web tool, presence awareness prevents conflicts and builds confidence.

### Implementation

- **WebSocket channel** per module: when a user opens a module, they join a presence channel
- **Presence broadcast**: user ID, display name, and cursor/scroll position shared with other participants
- **Editing indicator**: when a user begins editing an object, an icon appears next to that object for other viewers
- **Save conflict warning**: if two users edit the same object concurrently, the second saver receives a conflict warning with the option to merge or overwrite

## 8.23 Optional History Policy

### Motivation

Full attribute-level audit trails are essential for regulated environments post-baselining but create significant storage overhead for large modules during early drafting phases where every keystroke needn't be recorded.

### Module-Level Policy

| Policy | Behavior |
|--------|----------|
| `always` | Every mutation writes to `object_history` (current behavior, default) |
| `on_baseline` | History is written only when a baseline is created (snapshot all current versions) |
| `off` | No history tracking. Suitable for scratch/draft modules not under configuration management. |

### Implementation

- `module.history_policy` column: `TEXT DEFAULT 'always'`
- `insert_history()` function checks module policy before writing
- Policy change is itself audited (who changed the policy and when)
- Baselines in `on_baseline` mode: baseline creation triggers a bulk `INSERT INTO object_history` for all objects in the module, creating version snapshots
- Upgrading from `off` → `always`: does not retroactively create history; begins tracking from the policy change forward

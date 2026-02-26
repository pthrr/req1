# Functional Requirements

## 1. Data Model — FR-1xx

### Workspace

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-100 | The system SHALL provide a workspace entity as the top-level organizational container. | SHALL | Implemented |
| FR-101 | Each workspace SHALL have a unique name and optional description. | SHALL | Implemented |
| FR-102 | Workspaces SHALL isolate data: users with access to workspace A SHALL NOT see workspace B data unless explicitly granted. | SHALL | Planned |
| FR-103 | The system SHALL support CRUD operations on workspaces via REST API. | SHALL | Implemented |

### Project

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-110 | The system SHALL provide a project entity contained within a workspace. | SHALL | Implemented |
| FR-111 | Each project SHALL have a unique name (within its workspace) and optional description. | SHALL | Implemented |
| FR-112 | The system SHALL support CRUD operations on projects via REST API. | SHALL | Implemented |

### Module

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-120 | The system SHALL provide a module entity as a hierarchical container of objects (requirements). | SHALL | Implemented |
| FR-121 | Each module SHALL belong to exactly one project. | SHALL | Implemented |
| FR-122 | Each module SHALL have a configurable numbering scheme: prefix, separator, and digit count. | SHALL | Implemented |
| FR-123 | The system SHALL support creating a module from a template, copying attribute definitions, scripts, and object types from the source module. | SHALL | Implemented |
| FR-124 | Each module SHALL support a configurable default classification for new objects. | SHALL | Implemented |
| FR-125 | Each module SHALL support a configurable list of required attributes. | SHALL | Implemented |
| FR-126 | The system SHALL support CRUD operations on modules via REST API. | SHALL | Implemented |
| FR-127 | The system SHALL support soft deletion of modules with recoverability. | SHOULD | Planned |

### Object

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-130 | The system SHALL provide an object entity as the atomic traceable unit within a module. | SHALL | Implemented |
| FR-131 | Each object SHALL have a heading (short title) and an optional body (long description). | SHALL | Implemented |
| FR-132 | Each object SHALL have a classification: normative, informative, or heading. | SHALL | Implemented |
| FR-133 | Objects SHALL be hierarchically structured via parent-child relationships within a module. | SHALL | Implemented |
| FR-134 | Each object SHALL have a position field determining display order among siblings. | SHALL | Implemented |
| FR-135 | Each object SHALL have a computed outline level derived from its hierarchy depth. | SHALL | Implemented |
| FR-136 | The system SHALL support soft deletion of objects (recoverable via `deleted_at` timestamp). | SHALL | Implemented |
| FR-137 | The system SHALL support listing objects with a filter to include or exclude soft-deleted objects. | SHALL | Implemented |
| FR-138 | Each object SHALL support an optional `references_` field (JSONB) for external file/URL references. | SHALL | Implemented |
| FR-139 | The system SHALL support CRUD operations on objects via REST API, including pagination, filtering, and full-text search. | SHALL | Implemented |

---

## 2. Attributes & Object Types — FR-2xx

### Attribute Definitions

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-200 | The system SHALL support user-defined attribute definitions scoped to a module. | SHALL | Implemented |
| FR-201 | Each attribute definition SHALL specify a data type from: string, integer, float, boolean, enum, date, rich_text, user_ref. | SHALL | Implemented |
| FR-202 | Enum attribute definitions SHALL support single-select and multi-select modes. | SHALL | Implemented |
| FR-203 | Enum attribute definitions SHALL define a list of allowed values. | SHALL | Implemented |
| FR-204 | Attribute definitions SHALL support an optional default value. | SHOULD | Planned |
| FR-205 | Attribute values SHALL be stored as JSONB on the object entity, keyed by attribute definition ID. | SHALL | Implemented |
| FR-206 | The system SHALL support CRUD operations on attribute definitions via REST API. | SHALL | Implemented |
| FR-207 | Attribute definitions SHOULD support dependent enumerations where allowed values of one enum are filtered based on another enum's selection. | SHOULD | Planned |

### Object Types

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-210 | The system SHALL support user-defined object types with a name, default classification, required attributes, and attribute schema. | SHALL | Implemented |
| FR-211 | Each object MAY reference an object type via a foreign key. | SHALL | Implemented |
| FR-212 | When an object references a type, the system SHALL enforce the type's attribute schema on create and update. | SHALL | Implemented |
| FR-213 | The system SHALL support CRUD operations on object types via REST API. | SHALL | Implemented |
| FR-214 | The UI SHALL provide a type selector when creating new objects. | SHALL | Implemented |
| FR-215 | Object types SHOULD support a configurable form layout defining which attributes appear, in what order, grouped into sections. | SHOULD | Planned |

---

## 3. Links & Traceability — FR-3xx

### Link Types

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-300 | The system SHALL support user-defined link types with a name and optional description. | SHALL | Implemented |
| FR-301 | Default link types SHALL be seeded: satisfies, derives-from, verifies, mitigates, implements. | SHALL | Implemented |
| FR-302 | The system SHALL support CRUD operations on link types via REST API. | SHALL | Implemented |

### Links

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-310 | The system SHALL support directed links between two objects, optionally across modules. | SHALL | Implemented |
| FR-311 | Each link SHALL reference a link type. | SHALL | Implemented |
| FR-312 | Links SHALL support custom attributes (stored as JSONB). | SHALL | Implemented |
| FR-313 | The system SHALL support CRUD operations on links via REST API. | SHALL | Implemented |

### Suspect Detection

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-320 | The system SHALL automatically flag a link as "suspect" when the source object's content fingerprint changes after the link was created or last cleared. | SHALL | Implemented |
| FR-321 | The system SHALL support manual resolution (clearing) of suspect flags via API and CLI. | SHALL | Implemented |
| FR-322 | The traceability matrix SHALL visually indicate suspect links. | SHALL | Implemented |

### Traceability Matrix

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-330 | The system SHALL provide a cross-module traceability matrix showing links between objects of a source module and a target module. | SHALL | Implemented |
| FR-331 | The traceability matrix SHALL support filtering by link type. | SHALL | Implemented |

### Impact Analysis

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-340 | The system SHALL provide impact analysis via BFS graph traversal from a given object. | SHALL | Implemented |
| FR-341 | Impact analysis SHALL support configurable direction: forward, backward, or both. | SHALL | Implemented |
| FR-342 | Impact analysis SHALL support configurable maximum traversal depth. | SHALL | Implemented |

### Coverage Metrics

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-350 | The system SHALL compute coverage metrics per module: percentage of objects with upstream and downstream links. | SHALL | Implemented |
| FR-351 | Coverage metrics SHALL support filtering by link type. | SHALL | Implemented |

---

## 4. Baselines & Versioning — FR-4xx

### Object Versioning

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-400 | The system SHALL increment an object's version counter on every mutation. | SHALL | Implemented |
| FR-401 | The system SHALL store a full attribute snapshot in the object history table for every version. | SHALL | Implemented |
| FR-402 | The system SHALL record the change type (create, modify, delete) for each history entry. | SHALL | Implemented |
| FR-403 | The system SHALL provide an API endpoint to retrieve the version history of an object. | SHALL | Implemented |

### Content Fingerprinting

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-410 | The system SHALL compute a SHA-256 content fingerprint for each object based on its heading, body, and attribute values. | SHALL | Implemented |
| FR-411 | The content fingerprint SHALL be updated on every object mutation. | SHALL | Implemented |
| FR-412 | The content fingerprint SHALL be used for suspect link detection (FR-320). | SHALL | Implemented |

### Baselines

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-420 | The system SHALL support creating named, immutable baselines of a module. | SHALL | Implemented |
| FR-421 | A baseline SHALL be a set of (object_id, version) pointers capturing the exact state of all objects at baseline creation time. | SHALL | Implemented |
| FR-422 | Baselines SHALL be created atomically in a single database transaction. | SHALL | Implemented |
| FR-423 | Baseline entries SHALL NOT be modifiable after creation. | SHALL | Implemented |
| FR-424 | The system SHALL support CRUD operations on baselines via REST API (create, read, delete; no update). | SHALL | Implemented |

### Baseline Diffing

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-430 | The system SHALL support structured diffing between two baselines of the same module. | SHALL | Implemented |
| FR-431 | The diff SHALL classify objects as: added, removed, modified, or unchanged. | SHALL | Implemented |
| FR-432 | For modified objects, the diff SHALL provide word-level inline differences for heading and body fields. | SHALL | Implemented |
| FR-433 | For modified objects, the diff SHALL provide attribute-level comparison. | SHALL | Implemented |

### Baseline Sets

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-440 | The system SHALL support baseline sets: named groupings of baselines across modules representing a consistent system state. | SHALL | Partial |
| FR-441 | The system SHALL support CRUD operations on baseline sets via REST API. | SHALL | Implemented |
| FR-442 | The system SHOULD support cross-module baseline diff within a baseline set. | SHOULD | Planned |

---

## 5. Views, Search & Filtering — FR-5xx

### Saved Views

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-500 | The system SHALL support saved views per module, storing column configuration, filter configuration, and sort configuration. | SHALL | Implemented |
| FR-501 | The system SHALL support CRUD operations on views via REST API. | SHALL | Implemented |
| FR-502 | The UI SHALL allow switching between saved views via a dropdown. | SHALL | Implemented |
| FR-503 | The UI SHALL auto-apply the default view when opening a module. | SHALL | Implemented |

### Full-Text Search

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-510 | The system SHALL support full-text search within a module using PostgreSQL tsvector and GIN indexes. | SHALL | Implemented |
| FR-511 | Full-text search SHALL index object heading and body fields. | SHALL | Implemented |
| FR-512 | Full-text search results SHALL be ranked by relevance. | SHALL | Implemented |
| FR-513 | The system SHOULD support full-text search across all modules within a project. | SHOULD | Planned |

### Object Filtering

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-520 | The system SHALL support filtering objects by classification. | SHALL | Implemented |
| FR-521 | The system SHALL support filtering objects by review status (needs_review). | SHALL | Implemented |
| FR-522 | The system SHALL support pagination (limit, offset) on object listings. | SHALL | Implemented |
| FR-523 | The system SHOULD support multi-column sort on object listings. | SHOULD | Planned |

---

## 6. Access Control & Authentication — FR-6xx

### Users

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-600 | The system SHALL provide a user entity with email, display name, role, and active status. | SHALL | Implemented |
| FR-601 | The system SHALL support CRUD operations on users via REST API. | SHALL | Implemented |

### Authentication

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-610 | The system SHALL support authentication via OpenID Connect (OIDC) for corporate SSO. | SHALL | Planned |
| FR-611 | The system SHALL support local authentication (username + password) as a fallback. | SHALL | Planned |
| FR-612 | Passwords SHALL be hashed using argon2. | SHALL | Planned |
| FR-613 | The system SHALL issue JWT tokens for API authentication. | SHALL | Planned |

### Authorization

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-620 | The system SHALL enforce role-based access control (RBAC) with roles: admin, editor, reviewer, viewer. | SHALL | Planned |
| FR-621 | Roles SHALL be scoped to workspace, project, or module level. | SHALL | Planned |
| FR-622 | The system SHOULD support field-level permissions restricting visibility of specific attributes by role. | SHOULD | Planned |
| FR-623 | The system SHALL populate `changed_by` and `reviewed_by` fields from the authenticated user context. | SHALL | Planned |

### Electronic Signatures

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-630 | The system SHALL support electronic signatures on configured workflow transitions requiring re-authentication. | SHALL | Planned |
| FR-631 | Each e-signature SHALL record: signer identity, object identity, object version, transition name, meaning statement, timestamp, and a signature hash. | SHALL | Planned |
| FR-632 | The system SHALL enforce the four-eyes principle: the signer SHALL NOT be the same user as the object's last modifier when configured. | SHALL | Planned |
| FR-633 | E-signature records SHALL be immutable (append-only, never updated or deleted). | SHALL | Planned |

---

## 7. Scripting Engine — FR-7xx

### Script Management

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-700 | The system SHALL support Lua 5.4 scripts as first-class entities stored per module. | SHALL | Implemented |
| FR-701 | Each script SHALL have a name, body, description, and type (trigger, layout, action). | SHALL | Implemented |
| FR-702 | The system SHALL support CRUD operations on scripts via REST API. | SHALL | Implemented |

### Script Types

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-710 | Trigger scripts SHALL execute on events: pre_save, pre_delete, validate. | SHALL | Implemented |
| FR-711 | Layout scripts SHALL compute display values for virtual columns in the grid. | SHALL | Implemented |
| FR-712 | Action scripts SHALL support batch operations executed on demand via API. | SHALL | Implemented |

### Script Execution

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-720 | The system SHALL execute scripts in a sandboxed Lua environment with no file I/O, network access, or OS calls. | SHALL | Implemented |
| FR-721 | Script execution SHALL be subject to configurable memory and CPU time limits. | SHALL | Implemented |
| FR-722 | The system SHALL provide a `req1.*` API table to scripts for querying and mutating objects, links, and modules. | SHALL | Implemented |
| FR-723 | The system SHALL support dry-run (test) execution of scripts without persisting changes. | SHALL | Implemented |
| FR-724 | All data mutations made by scripts SHALL be attributed to the executing user in the audit trail. | SHALL | Implemented |

### Script Scheduling

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-730 | The system SHOULD support CRON-based scheduling for scripts. | SHOULD | Planned |
| FR-731 | Scheduled script executions SHALL be logged with status, duration, and output. | SHOULD | Planned |

---

## 8. Import / Export & Interchange — FR-8xx

### ReqIF

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-800 | The system SHALL support importing requirements from OMG ReqIF 1.2 files. | SHALL | Planned |
| FR-801 | ReqIF import SHALL map SPEC-TYPES to attribute definitions, SPEC-OBJECTS to objects, and SPEC-RELATIONS to links. | SHALL | Planned |
| FR-802 | The system SHALL support exporting modules to OMG ReqIF 1.2 format. | SHALL | Planned |
| FR-803 | ReqIF round-trip SHALL preserve all standard attribute types (string, integer, real, date, enum, XHTML). | SHALL | Planned |

### CSV

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-810 | The system SHALL support exporting objects to CSV format. | SHALL | Planned |
| FR-811 | The system SHALL support importing objects from CSV format. | SHALL | Planned |

### Excel

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-820 | The system SHALL support exporting objects to XLSX format with attribute columns and a metadata sheet. | SHALL | Planned |
| FR-821 | The system SHALL support importing objects from XLSX format with column-to-attribute mapping. | SHALL | Planned |
| FR-822 | Excel round-trip SHALL preserve object IDs across export, external edit, and reimport. | SHALL | Planned |

### Word

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-830 | The system SHALL support exporting modules to Word (.docx) format as structured documents. | SHALL | Planned |
| FR-831 | The system SHOULD support importing requirements from Word documents with heading-to-object mapping. | SHOULD | Planned |

### YAML

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-840 | The system SHOULD support exporting modules to YAML format for docs-as-code workflows. | SHOULD | Planned |

---

## 9. Review, Collaboration & Workflow — FR-9xx

### Review Packages

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-900 | The system SHALL support review packages grouping objects for formal review. | SHALL | Partial |
| FR-901 | Each review package SHALL support assigning reviewers via review assignments. | SHALL | Partial |
| FR-902 | The system SHALL support CRUD operations on review packages and assignments via REST API. | SHALL | Implemented |

### Object Review

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-910 | The system SHALL track review status per object: reviewed or needs_review. | SHALL | Implemented |
| FR-911 | The system SHALL record the reviewer identity and timestamp when an object is reviewed. | SHALL | Implemented |
| FR-912 | When a reviewed object is modified, its review status SHALL reset to needs_review. | SHALL | Implemented |

### Voting

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-920 | The system SHALL support formal voting on review artifacts: approve, reject, or abstain per reviewer. | SHALL | Planned |
| FR-921 | The system SHALL provide a voting dashboard aggregating results per review package. | SHALL | Planned |

### Comments

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-930 | The system SHALL support threaded comments per object. | SHALL | Implemented |
| FR-931 | Comments SHALL support resolve/unresolve status. | SHALL | Implemented |
| FR-932 | The system SHALL support CRUD operations on comments via REST API. | SHALL | Implemented |
| FR-933 | Comments SHOULD support @mentioning users with notification delivery. | SHOULD | Planned |

### Change Proposals

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-940 | The system SHALL support change proposals with diff data describing proposed modifications. | SHALL | Partial |
| FR-941 | Change proposals SHALL support status transitions: draft, submitted, approved, rejected. | SHALL | Planned |
| FR-942 | Approved proposals SHOULD be applicable as batch updates to the target objects. | SHOULD | Planned |

### Lifecycle Management

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-950 | The system SHALL support configurable lifecycle models defining states and allowed transitions. | SHALL | Planned |
| FR-951 | Each lifecycle state SHALL have a name, color, and terminal/initial flags. | SHALL | Planned |
| FR-952 | Each lifecycle transition SHALL define the from-state, to-state, and optionally a required role. | SHALL | Planned |
| FR-953 | Modules SHALL reference a lifecycle model; all objects in the module follow that lifecycle. | SHALL | Planned |
| FR-954 | Object types MAY override the module's default lifecycle model. | MAY | Planned |
| FR-955 | State transitions SHALL be audited in the object history. | SHALL | Planned |

### Notifications

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-960 | The system SHOULD support webhook notifications on object changes. | SHOULD | Planned |
| FR-961 | The system SHOULD support real-time presence awareness showing concurrent editors of a module. | SHOULD | Planned |

---

## 10. Publishing & Reporting — FR-10xx

### HTML Publishing

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-1000 | The system SHALL support publishing a module to HTML using Minijinja templates. | SHALL | Implemented |
| FR-1001 | HTML output SHALL include configurable outline numbering (prefix, separator, digits). | SHALL | Implemented |
| FR-1002 | Published HTML SHOULD include deep links back to the req1 UI for each object. | SHOULD | Planned |

### Additional Formats

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-1010 | The system SHALL support publishing to Markdown format. | SHALL | Planned |
| FR-1011 | The system SHALL support publishing to PDF format. | SHALL | Planned |
| FR-1012 | The system SHOULD support publishing to LaTeX format. | SHOULD | Planned |
| FR-1013 | The system SHOULD support custom template upload per module. | SHOULD | Planned |

### Dashboards

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-1020 | The system SHOULD support configurable dashboard pages with widget layouts. | SHOULD | Planned |
| FR-1021 | Widgets SHOULD include: coverage chart, suspect link count, lifecycle state distribution, review status. | SHOULD | Planned |
| FR-1022 | Dashboards SHOULD support querying across multiple modules. | SHOULD | Planned |

---

## 11. Integration Interfaces — FR-11xx

### REST API

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-1100 | The system SHALL expose a REST API covering all operations the UI can perform. | SHALL | Implemented |
| FR-1101 | All API endpoints SHALL accept and return JSON. | SHALL | Implemented |
| FR-1102 | All entity IDs SHALL be UUIDs (v7). | SHALL | Implemented |
| FR-1103 | The API SHALL follow consistent CRUD patterns across all resource types. | SHALL | Implemented |

### CLI

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-1110 | The system SHALL provide a CLI client (`req1-cli`) for headless automation. | SHALL | Implemented |
| FR-1111 | The CLI SHALL support: list, create, update, delete operations on core entities. | SHALL | Implemented |
| FR-1112 | The CLI SHALL support validate, review, publish, and resolve-suspect commands. | SHALL | Implemented |
| FR-1113 | The CLI SHALL support output formats: table (default), tree, and JSON. | SHALL | Implemented |
| FR-1114 | The CLI `validate` command SHALL exit with code 1 on errors, enabling CI pipeline integration. | SHALL | Implemented |

### OSLC

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-1120 | The system SHALL implement an OSLC Core 3.0 service provider exposing requirement objects as OSLC-RM 2.1 resources. | SHALL | Planned |
| FR-1121 | The OSLC provider SHALL support a service provider catalog, delegated selection/creation dialogs, and query capability. | SHALL | Planned |
| FR-1122 | The system SHALL support creating links to external OSLC resources (stored with `external_uri`). | SHALL | Planned |
| FR-1123 | Suspect detection SHALL apply to external OSLC links when the source object is modified. | SHALL | Planned |

### ReqIF Interchange

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-1130 | ReqIF import/export is specified in FR-800 through FR-803. | — | — |

### MCP (Model Context Protocol)

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-1140 | The system SHOULD expose an MCP server enabling AI assistants to query and modify requirements data. | SHOULD | Planned |
| FR-1141 | MCP tool calls SHALL be subject to the same RBAC as REST API calls. | SHALL | Planned |

### SysML v2

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| FR-1150 | The system SHOULD support importing requirements from SysML v2 JSON (`RequirementUsage`, `SatisfyRequirementUsage`). | SHOULD | Planned |
| FR-1151 | The system SHOULD support exporting modules as SysML v2 JSON requirement elements. | SHOULD | Planned |

---

## Requirement Count Summary

| Status | Count |
|--------|-------|
| Implemented | 109 |
| Partial | 4 |
| Planned | 63 |
| **Total** | **176** |

# TODO — Gap Analysis vs Doorstop, DOORS Classic, PREEvision & Polarion

## Legend

- `[x]` done
- `[-]` partial / stub
- `[ ]` not started

---

## Done

- [x] Workspace > Project > Module > Object hierarchy
- [x] Per-object versioning + history table with change_type audit
- [x] Content fingerprinting (SHA-256) + review stamps
- [x] Suspect link auto-detection + manual resolution
- [x] Named link types (user-defined)
- [x] Cross-module directed links with attributes
- [x] Typed attribute definitions (string, int, float, bool, enum, date, rich_text, user_ref)
- [x] Classification: normative / informative / heading
- [x] JavaScript scripting — triggers (pre_save, pre_delete, validate), actions, layout stub
- [x] Module baselines with snapshot diffing (word-level inline diff for heading/body/attributes)
- [x] Validation service — built-in structural rules + JavaScript custom rules + required_attributes
- [x] HTML publishing (Minijinja template with prefix/separator/digits context)
- [x] Full-text search (PostgreSQL tsvector)
- [x] Traceability matrix (cross-module, suspect indicators)
- [x] Web UI: AG Grid inline editing, tree expand/collapse, filters, search
- [x] Object tree sidebar panel (240px, collapsible, click-to-scroll-grid)
- [x] Baseline diff viewer with expandable modified rows and word-level diff
- [x] Module config: prefix, separator, digits, required_attributes, default_classification
- [x] Settings tab in frontend
- [x] CLI: list/create/update/delete, validate, review, publish, import, export, tree view, JSON output
- [x] Data model: external references (JSONB on object)
- [x] Data model: saved views table (column_config, filter_config, sort_config)
- [x] Data model: object types table + FK on object
- [x] Data model: comments table
- [x] Data model: app_user table
- [x] Data model: review_package + review_assignment tables
- [x] Data model: change_proposal table
- [x] Data model: baseline_set table + FK on baseline
- [x] Data model: attachment table
- [x] Data model: soft delete (deleted_at on object)
- [x] Data model: multi-value enums (multi_select on attribute_definition)
- [x] CRUD services + routes for: views, object_types, comments, app_users, review_packages, review_assignments, change_proposals, baseline_sets
- [x] Object soft-delete service method + include_deleted filter in list
- [x] Backend integration tests (58 tests covering all CRUD routes, validation, impact, coverage, templates, publish, soft delete)
- [x] Playwright E2E tests (47 tests covering sidebar CRUD, tabs, objects, history, links, baselines, scripts, validation, types, settings, comments, impact, coverage, templates, views, attributes, detail panel, split view, breadcrumbs, reviews, proposals, references, dark mode, search highlighting)
- [x] Taskfile: `task test:backend`, `task test:e2e` (full lifecycle), `task ci` (fmt + clippy + tsc + backend + E2E)
- [x] Taskfile: `task dev` resets DB with fresh migrations on every start
- [x] Object detail panel (ObjectDetailPanel) with markdown preview, split edit/preview
- [x] Keyboard shortcuts: Enter=sibling, Tab=child, Alt+arrows=move/indent
- [x] Split-pane view: tree + detail form toggle
- [x] Breadcrumb navigation with clickable ancestor path
- [x] Search result highlighting in tree (yellow highlight on matches)
- [x] Link navigation: click linked object to jump, cross-module support, back button
- [x] Format selector dropdown for publish (HTML, Markdown, LaTeX, Plain Text)
- [x] In-app document preview (PublishPreviewPanel with iframe + print)
- [x] Batch review button + bulk operations (multi-select batch delete/classify/review)
- [x] Review tab: ReviewPanel with packages, assignments, status transitions (draft→approved)
- [x] Review dashboard: SVG bar chart (reviewed/unreviewed %) + status summary
- [x] Review diff: ReviewDiffPanel comparing reviewed vs current version
- [x] Change proposals UI: ChangeProposalPanel with create/view/approve/reject + batch apply
- [x] Baseline sets UI: group baselines into named versioned sets with filtering
- [x] References panel: ReferencesPanel modal for managing external refs per object
- [x] Activity feed: collapsible changelog per module
- [x] Dark mode: ThemeContext with light/dark palettes, localStorage, header toggle

---

## Priority 1 — Table Stakes (Doorstop has these, we don't)

### Import / Export

Doorstop supports CSV, TSV, XLSX, YAML for both import and export. We have zero.

- [x] CSV export (CLI `req1 publish --format csv` + API endpoint)
- [x] CSV import (CLI `req1 import --format csv` + API endpoint + frontend Import CSV button)
- [x] ReqIF import/export (XML library, entity mapping, API endpoints, CLI commands) — Doorstop lacks this, so it's a differentiator
- [x] YAML export (full round-trip via `req1 publish --format yaml` + API endpoint)

### Additional Publish Formats

Doorstop publishes to HTML, LaTeX, Markdown, and plain text. We have HTML only.

- [x] Markdown publish (`?format=md`) — enables "docs-as-code" workflows
- [x] LaTeX publish (`?format=latex`) — regulated industries (DO-178C, ISO 26262) require this
- [x] Plain text publish (`?format=txt`)
- [x] PDF export (wkhtmltopdf or weasyprint via subprocess)
- [x] Custom template upload per module (publish_template column on module)
- [x] PlantUML diagram rendering in published output (Doorstop supports this)

### Object Reordering UX

Doorstop has CLI `reorder` and GUI indent/dedent. Our position/parent_id editing is too low-level.

- [x] Drag-and-drop reorder in grid
- [x] Indent / dedent buttons (reparent object one level up/down)
- [x] Move up / move down within siblings
- [x] CLI `reorder` command

### External References

- [x] `references_` JSONB field on objects (data model)
- [x] SHA integrity checking for referenced files
- [x] UI panel to manage references per object (ReferencesPanel modal: add/edit/remove URL/file/document refs)

---

## Priority 2 — DOORS-Inspired Features (Competitive Differentiation)

### Saved Views / Column Layouts

- [x] `View` entity + CRUD service + API routes
- [x] UI: save/load/switch views dropdown in Objects tab (ViewBar component)
- [x] Apply view config to AG Grid on load (auto-applies default view on gridReady)

### Object Types / Schemas

- [x] `ObjectType` entity + CRUD service + API routes (with default_classification, required_attributes, attribute_schema)
- [x] Objects reference a type via `object_type_id` FK
- [x] Object create/update: enforce type constraints (attribute_schema validation)
- [x] Type selector when creating objects in UI
- [x] Object Types CRUD panel (Types tab in module detail)
- [x] Module templates (create module from template with copied attr defs + scripts + object types)

### Computed Columns (Layout Scripts)

DOORS has Layout DXL — code in a column definition that computes display values.

- [x] Complete `layout` script type (engine + batch layout endpoint)
- [x] JavaScript function returns a display value for a column
- [x] UI: layout-script-backed columns in AG Grid (auto-fetched from enabled layout scripts)
- [x] Seed layout script examples (Link Count, Classification Badge, Has Body)

### Impact Analysis & Coverage

- [x] Transitive link traversal: BFS with direction (forward/backward/both) + max depth
- [x] Coverage metrics: % of objects with upstream/downstream links (backend endpoint + frontend widget)
- [x] Graph visualization of link chains (D3 force-directed graph in impact panel)
- [x] Impact analysis panel in the UI (modal with depth-grouped results + graph toggle)

### Discussion / Comments

- [x] `Comment` entity + CRUD service + API routes
- [x] UI: comment thread per object (CommentPanel modal)
- [x] Resolve/unresolve comments

---

## Priority 3 — Enterprise Features

### Authentication & Access Control

- [x] `AppUser` entity + CRUD service + API routes (email, display_name, role, active)
- [x] User authentication (JWT or session-based)
- [x] Per-workspace roles: admin, editor, viewer
- [x] Per-module permissions: read, write, admin
- [x] Populate `reviewed_by` / `changed_by` from auth context
- [x] Audit trail with authenticated identity

### Formal Review Workflow

- [x] `ReviewPackage` + `ReviewAssignment` entities + CRUD services + API routes
- [x] Multi-reviewer sign-off tracking (UI) — ReviewPanel with expandable assignments per package, reviewer dropdown
- [x] Review status dashboard — ReviewDashboard with SVG bar chart (reviewed/unreviewed %) + package status summary
- [x] Approval workflow (status transitions: draft → open → in_review → approved/rejected) — ReviewPanel status flow with transition buttons

### Change Proposals

- [x] `ChangeProposal` entity + CRUD service + API routes (with diff_data JSONB)
- [x] UI: create/view/approve/reject proposals — ChangeProposalPanel with diff details, status transitions
- [x] Apply approved proposal as batch update — ChangeProposalPanel "Apply" button parses diff_data and applies changes

### Baseline Sets

- [x] `BaselineSet` entity + CRUD service + API routes
- [x] `baseline_set_id` FK on baseline
- [x] UI: group baselines into named versioned sets — BaselinePanel with baseline sets CRUD, filtering by set
- [x] Cross-module baseline diff

### Concurrency / Collaboration

- [x] Optimistic locking (version-based conflict detection on update, HTTP 409 on conflict)
- [x] Conflict detection on concurrent edits (expected_version field in update API)
- [x] Notifications / webhooks on object changes

### Attachments

- [x] `Attachment` entity + data model (file_name, content_type, size_bytes, storage_path, sha256)
- [x] File upload/download endpoints (multipart upload, SHA-256, filesystem storage)
- [x] UI: attachment list per object (AttachmentPanel with upload/download/delete)

---

## Priority 4 — UX Polish

### Object Editing

- [x] Object detail panel / form view — ObjectDetailPanel modal with heading, body, classification, attributes, type selector
- [x] Live markdown preview while editing body — split edit/preview mode in ObjectDetailPanel using react-markdown
- [x] Keyboard shortcuts: Enter=add sibling, Tab=add child, Alt+arrows=move/indent/dedent
- [x] Split-pane view: tree + detail form — toggle between grid view and split view (tree + ObjectDetailPanel inline)

### Navigation

- [x] Hyperlink navigation between linked objects — click linked object badge to jump to target, cross-module support
- [x] Back button after link navigation — navigation history stack with back button
- [x] Breadcrumb within module — clickable ancestor path when object selected
- [x] Search result highlighting in tree — yellow highlight on matching objects in ObjectTree

### Review UX

- [x] Visual diff: reviewed version vs current version per object — ReviewDiffPanel comparing last two versions
- [x] Batch review button in UI — "Review All" button + bulk review in multi-select toolbar
- [x] Review status dashboard (bar chart: reviewed/unreviewed) — ReviewDashboard with SVG bar chart

### Publishing UX

- [x] In-app document preview — PublishPreviewPanel with iframe + print button
- [x] Template editor in UI
- [x] Format selector dropdown — HTML/Markdown/LaTeX/PlainText dropdown with enabled/disabled states

### Miscellaneous

- [x] Activity feed / changelog per module — ActivityFeed collapsible panel showing recent changes
- [x] Bulk operations: multi-select -> batch delete/classify/review — multi-select toolbar with batch actions
- [x] Module copy / branch (clone objects + links with create_from_template copy_objects flag)
- [x] Full-text search across modules (GET /search with cross-module tsvector + module context)
- [x] Soft delete / recycle bin (deleted_at + soft_delete service method)
- [x] Multi-value enums (multi_select on attribute_definition)
- [x] Multi-column sort in object list query (comma-separated sort_by/sort_dir)
- [x] Default value auto-application on new objects (from attribute_definition.default_value)
- [x] Dark mode — ThemeContext with light/dark palettes, localStorage persistence, toggle in header

---

## Priority 5 — PREEvision IREQ Parity

Features identified from Vector PREEvision's Integrated Requirements Engineering (IREQ) that are missing or incomplete in req1.

### Lifecycle / Workflow Management

PREEvision has a configurable lifecycle state model on requirements (new → draft → in_review → approved → released) with color-coded status and organization-specific workflow transitions.

- [x] Configurable lifecycle model entity (states + allowed transitions)
- [x] Lifecycle state field on objects with color-coded display in grid
- [x] State transition enforcement (only allowed transitions)
- [x] Lifecycle templates per module or object type

### Use Case Diagrams

PREEvision models system functions from the user perspective with UML use-case diagrams.

- [x] Evaluate diagramming approach (embedded editor vs external tool link)
- [x] Use-case diagram entity linked to requirements
- [x] Diagram rendering in UI (e.g., Mermaid or PlantUML)

### Integrated Test Engineering & Management

PREEvision links requirements directly to test cases and tracks test execution. req1 has no test artifact model.

- [x] Test case entity (linked to requirements via requirement_ids JSONB)
- [x] Test execution entity (status, executor, timestamp, evidence)
- [x] Test coverage metrics (% requirements with linked passing tests)
- [x] Test status dashboard per module

### Reuse / Placeholder Mechanism

PREEvision allows requirements to be reused across modules as placeholders (embedded references that stay in sync with the source).

- [x] Placeholder / proxy object type (references a source object in another module)
- [x] Sync mechanism: placeholder reflects source object's current content
- [x] UI: distinguish placeholders from native objects visually
- [x] Break-link action to convert placeholder to independent copy

### Excel Import / Export

PREEvision supports Excel as a primary exchange format. Already on Priority 1 implicitly but elevated here.

- [x] Excel export (XLSX with attribute columns, metadata sheet)
- [x] Excel import (map columns to attributes, create/update objects)
- [x] Round-trip: preserve object IDs across export → edit → reimport

### Deep Links from Published Reports

PREEvision generates reports with hyperlinks back into the model.

- [x] Published HTML includes deep links to req1 UI for each object
- [x] Anchor-based navigation (object ID in URL fragment)

### Review Voting & Approval Workflow

PREEvision has formal voting projects where stakeholders vote on requirements, with a dedicated chat view for discussion.

- [x] Voting entity (vote per reviewer per artifact: approve / reject / abstain)
- [x] Voting dashboard (aggregated results per review package)
- [x] Chat/discussion view scoped to review package (not just per-object comments)
- [x] Status transitions on review packages (draft → open → in_review → approved / rejected) — implemented in ReviewPanel

### Rich Text Editor with Tables & Graphics

PREEvision has a full rich text editor with embedded tables and graphics for requirement descriptions.

- [x] TipTap rich text editor integration (already planned in solution strategy)
- [x] Table support in requirement body (TipTap table extension)
- [x] Image embedding in requirement body (upload + inline display)
- [x] Graphics / diagram embedding (paste or drag-and-drop)

---

## Priority 6 — Polarion ALM Parity

Features identified from Siemens Polarion ALM that are missing or incomplete in req1. Items already covered by PREEvision Priority 5 are not repeated.

### LiveDoc / Document View (Dual Nature)

Polarion's core innovation: LiveDocs are Word-like documents where every paragraph is simultaneously a traceable, workflow-controlled database object. req1 is purely object-based with no document view.

- [x] Document view entity (LiveDoc equivalent: ordered sequence of objects rendered as a continuous document)
- [x] WYSIWYG document editing mode (paragraph = object, preserving traceability)
- [x] Switch between document view and grid/object view for the same module
- [x] Document outline / navigator sidebar in document mode
- [x] Export document view to Word/PDF preserving formatting

### Electronic Signatures (E-Signatures)

Polarion supports FDA 21 CFR Part 11 compliant e-signatures: username + password confirmation on workflow transitions. Required for medical devices and pharma.

- [x] E-signature mechanism (re-authentication on critical transitions)
- [x] Signature audit record (user, timestamp, meaning, signature hash)
- [x] E-signature requirement configurable per workflow transition
- [x] Four-eyes principle enforcement (signer ≠ author)

### Word Import / Export (Round-Trip)

Polarion has a rule-based import wizard that maps Word paragraphs to requirements and supports export → edit → reimport.

- [x] Word (.docx) export from module (structured document with styles)
- [x] Word import wizard (map headings/paragraphs to object types and attributes)
- [x] Round-trip: track object IDs through export → external edit → reimport

### Form Layout Designer

Polarion allows admins to design per-type Work Item forms: which fields appear, in what order, grouped into sections.

- [x] Configurable object form layout per object type
- [x] Form section grouping for attributes
- [x] Form layout editor in admin UI

### Dependent Enumerations (Cascading Fields)

Polarion enum field values can depend on another enum field's selection (cascading dropdowns).

- [x] Dependent enum definitions (parent enum → child enum value filtering)
- [x] UI: cascading dropdown behavior in object editor

### Scheduled Scripts (CRON)

Polarion supports server-side scripts that run on configurable CRON schedules for automated validation, report generation, and cleanup.

- [x] Script scheduling (CRON expression per script)
- [x] Scheduled script execution engine (background job runner)
- [x] Execution log with status, duration, output

### Save Hooks / Interceptors

Polarion has Java-based hooks that execute before/after saving a Work Item for custom validation, auto-population, and cross-artifact updates.

- [x] Pre-save / post-save hook framework (extends JavaScript trigger system)
- [x] Hook ordering and priority
- [x] Hook failure blocks save with error message

### @Mentions in Comments

Polarion supports @mentioning users in comments with notification delivery.

- [x] @mention syntax in comments (parse `@username`)
- [x] User autocomplete dropdown on `@` input
- [x] Notification delivery to mentioned users

### Cross-Project Dashboards & Reporting

Polarion supports LiveReport pages with drag-and-drop widgets querying across multiple projects.

- [x] Dashboard entity with configurable widget layout
- [x] Cross-module/cross-project data aggregation in widgets
- [x] Widget library (coverage chart, suspect link count, lifecycle distribution, test status)
- [x] Dashboard export to PDF

### Compliance Project Templates

Polarion ships pre-configured project templates for regulated industries (IEC 62304, ISO 26262, DO-178C, Automotive SPICE).

- [x] Project template entity (workspace template with pre-configured modules, attribute defs, object types, lifecycle models, scripts)
- [x] Built-in templates for ISO 26262, DO-178C, IEC 62304
- [x] Template instantiation wizard

---

## Architecture Notes

### Lessons from Doorstop

1. **Zero infrastructure mode** — Doorstop works with just a git repo. Consider SQLite mode for local/offline use (`req1 init --local`).
2. **Git hooks for CI** — Doorstop ships pre-commit hooks. Our `req1 validate` already returns exit code 1 — document the CI integration pattern.
3. **One-file-per-item** — minimizes merge conflicts in distributed teams. Not applicable to our DB approach, but consider git-based export for offline workflows.
4. **Multiple publish formats are table-stakes** — Doorstop has 4 output formats. We need at least Markdown and LaTeX.

### Lessons from DOORS Classic

1. **Object type system** is the single biggest abstraction gap. Typed schemas > freeform attributes.
2. **Saved views** are essential for daily multi-user use. Different stakeholders need different views of the same module.
3. **Layout DXL / computed columns** — nobody else has this as an open-source feature. Completing our JavaScript layout scripts would be a differentiator.
4. **Link modules** — treating links as first-class entities enables separate access control and management.
5. **Impact analysis** is the killer feature of any traceability tool. We have the data, we need the visualization.

### Lessons from PREEvision IREQ

1. **Lifecycle management is table stakes** for enterprise RE tools. PREEvision's configurable state model with color-coded transitions is a core workflow feature, not a nice-to-have.
2. **Requirements don't stand alone** — PREEvision emphasizes that requirements interact with all other development artifacts. Our link model covers this partially, but placeholders/reuse and test integration go further.
3. **Integrated test management** — PREEvision treats test engineering as part of the RE workflow, not a separate tool concern. Test cases, execution, and coverage are first-class.
4. **Excel is the universal interchange** — while ReqIF is important for toolchain interop, Excel is what most stakeholders actually use for review and editing.
5. **Voting/approval is separate from comments** — PREEvision distinguishes discussion (chat) from formal decisions (votes). Our comment model needs a voting layer on top.
6. **Reports link back to the model** — published specs in PREEvision are not dead documents; they contain navigable links back into the live model.

### Lessons from Polarion ALM

1. **LiveDoc dual nature is the killer concept** — requirements that are simultaneously document paragraphs and traceable database objects. This bridges the gap between document-centric (Word/PDF deliverables) and object-centric (traceability, filtering, dashboards) workflows. Consider a document view mode for req1 modules.
2. **E-signatures are mandatory for regulated industries** — FDA 21 CFR Part 11 compliance requires re-authentication on critical transitions (not just click-to-approve). Medical device and pharma customers will require this.
3. **Word round-trip is as important as Excel** — many regulated processes require Word documents (SRS, SDD, SVP). Import from Word, export to Word, and round-trip without data loss are expected.
4. **Compliance templates accelerate adoption** — Polarion ships pre-configured project templates for ISO 26262, DO-178C, IEC 62304, Automotive SPICE. Having ready-to-use templates dramatically lowers the barrier to entry for regulated teams.
5. **Form layout configurability** — different object types need different form layouts. A safety requirement form looks different from a test case form. Admins need to configure this without code.
6. **Scheduled automation** — CRON-based script execution enables proactive data quality (nightly validation, stale link detection, metric computation). Our JavaScript engine needs a scheduler.
7. **Real-time collaboration awareness** — seeing who else is editing the same module prevents conflicts and builds confidence in a multi-user tool.
8. **Collections (cross-project baseline sets)** — our baseline_sets are per-module; Polarion's Collections span projects and freeze everything for compliance audits.

### Our Advantages Over All Four

1. **Modern stack** — Rust (safety, perf), React, PostgreSQL vs Python/Tkinter, legacy C++/Windows-only, Eclipse RCP (PREEvision), or Java/Jetty (Polarion).
2. **Lightweight deployment** — single Rust binary + PostgreSQL vs Polarion's heavy Java stack (SVN-backed, requires dedicated server infrastructure).
3. **Server-based with full CRUD web UI** — Doorstop web is read-only. DOORS is desktop-only. PREEvision is a thick Eclipse client. Polarion's web UI is capable but heavyweight.
4. **Structured baseline diffing** — word-level inline diff with attribute comparison. Neither Doorstop nor DOORS has this. PREEvision's versioning is model-level. Polarion diffs at field level but not word level.
5. **JavaScript scripting** — more powerful than Doorstop's validator-only plugins, cleaner than DXL, more accessible than PREEvision's Java extensions, and lighter than Polarion's Velocity/Java SDK.
6. **Cross-module named typed links** — better traceability model than Doorstop's single unnamed parent-child links.
7. **Open source (MIT)** — PREEvision and Polarion are expensive proprietary commercial software. No vendor lock-in, full auditability of the tool itself, no per-seat licensing.
8. **REST API first** — every operation available via API and CLI. PREEvision requires Java plugins. Polarion's REST API is secondary to its Java SDK.
9. **MCP / AI-native integration** — planned MCP server enables AI assistants as first-class consumers. Polarion X is Azure-locked; req1's approach is model-agnostic.

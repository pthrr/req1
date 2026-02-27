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
- [x] CLI: list/create/update/delete, validate, review, publish, tree view, JSON output
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
- [x] Playwright E2E tests (28 tests covering sidebar CRUD, tabs, objects, history, links, baselines, scripts, validation, types, settings, comments, impact, coverage, templates, views, attributes)
- [x] Taskfile: `task test:backend`, `task test:e2e` (full lifecycle), `task ci` (fmt + clippy + tsc + backend + E2E)
- [x] Taskfile: `task dev` resets DB with fresh migrations on every start

---

## Priority 1 — Table Stakes (Doorstop has these, we don't)

### Import / Export

Doorstop supports CSV, TSV, XLSX, YAML for both import and export. We have zero.

- [ ] CSV export (CLI `req1 export --format csv` + API endpoint)
- [ ] CSV import (CLI `req1 import --format csv` + API endpoint)
- [-] ReqIF import/export (crate `req1-reqif` exists as empty stub) — Doorstop lacks this too, so it's a differentiator
- [ ] YAML export (full round-trip, Doorstop's native format)

### Additional Publish Formats

Doorstop publishes to HTML, LaTeX, Markdown, and plain text. We have HTML only.

- [ ] Markdown publish (`?format=md`) — enables "docs-as-code" workflows
- [ ] LaTeX publish (`?format=latex`) — regulated industries (DO-178C, ISO 26262) require this
- [ ] Plain text publish (`?format=txt`)
- [ ] PDF export (LaTeX pipeline or headless browser)
- [ ] Custom template upload per module (currently single hardcoded template)
- [ ] PlantUML diagram rendering in published output (Doorstop supports this)

### Object Reordering UX

Doorstop has CLI `reorder` and GUI indent/dedent. Our position/parent_id editing is too low-level.

- [ ] Drag-and-drop reorder in grid
- [ ] Indent / dedent buttons (reparent object one level up/down)
- [ ] Move up / move down within siblings
- [ ] CLI `reorder` command

### External References

- [x] `references_` JSONB field on objects (data model)
- [ ] SHA integrity checking for referenced files
- [ ] UI panel to manage references per object

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
- [ ] User authentication (JWT or session-based)
- [ ] Per-workspace roles: admin, editor, viewer
- [ ] Per-module permissions: read, write, admin
- [ ] Populate `reviewed_by` / `changed_by` from auth context
- [ ] Audit trail with authenticated identity

### Formal Review Workflow

- [x] `ReviewPackage` + `ReviewAssignment` entities + CRUD services + API routes
- [ ] Multi-reviewer sign-off tracking (UI)
- [ ] Review status dashboard
- [ ] Approval workflow (status transitions: draft → open → in_review → approved/rejected)

### Change Proposals

- [x] `ChangeProposal` entity + CRUD service + API routes (with diff_data JSONB)
- [ ] UI: create/view/approve/reject proposals
- [ ] Apply approved proposal as batch update

### Baseline Sets

- [x] `BaselineSet` entity + CRUD service + API routes
- [x] `baseline_set_id` FK on baseline
- [ ] UI: group baselines into named versioned sets
- [ ] Cross-module baseline diff

### Concurrency / Collaboration

- [ ] Optimistic locking (ETag / version check on update)
- [ ] Conflict detection on concurrent edits
- [ ] Notifications / webhooks on object changes

### Attachments

- [x] `Attachment` entity + data model (file_name, content_type, size_bytes, storage_path, sha256)
- [ ] File upload/download endpoints (multipart)
- [ ] UI: attachment list per object

---

## Priority 4 — UX Polish

### Object Editing

- [ ] Object detail panel / form view (not just in-grid editing)
- [ ] Live markdown preview while editing body
- [ ] Keyboard shortcuts: add sibling (Enter), add child (Tab), move (Alt+arrows)
- [ ] Split-pane view: tree + detail form

### Navigation

- [ ] Hyperlink navigation between linked objects (click link -> jump to target object)
- [ ] Back button after link navigation
- [ ] Breadcrumb within module (level 1 > 1.1 > 1.1.2)
- [ ] Search result highlighting in tree

### Review UX

- [ ] Visual diff: reviewed version vs current version per object
- [ ] Batch review button in UI
- [ ] Review status dashboard (bar chart: reviewed/unreviewed)

### Publishing UX

- [ ] In-app document preview (rendered HTML in iframe)
- [ ] Template editor in UI
- [ ] Format selector dropdown

### Miscellaneous

- [ ] Activity feed / changelog per module
- [ ] Bulk operations: multi-select -> batch delete/classify/review
- [ ] Module copy / branch (fork a module for parallel development)
- [ ] Full-text search across modules (currently per-module only)
- [x] Soft delete / recycle bin (deleted_at + soft_delete service method)
- [x] Multi-value enums (multi_select on attribute_definition)
- [ ] Multi-column sort in object list query
- [ ] Default value auto-application on new objects
- [ ] Dark mode

---

## Priority 5 — PREEvision IREQ Parity

Features identified from Vector PREEvision's Integrated Requirements Engineering (IREQ) that are missing or incomplete in req1.

### Lifecycle / Workflow Management

PREEvision has a configurable lifecycle state model on requirements (new → draft → in_review → approved → released) with color-coded status and organization-specific workflow transitions.

- [ ] Configurable lifecycle model entity (states + allowed transitions)
- [ ] Lifecycle state field on objects with color-coded display in grid
- [ ] State transition enforcement (only allowed transitions)
- [ ] Lifecycle templates per module or object type

### Use Case Diagrams

PREEvision models system functions from the user perspective with UML use-case diagrams.

- [ ] Evaluate diagramming approach (embedded editor vs external tool link)
- [ ] Use-case diagram entity linked to requirements
- [ ] Diagram rendering in UI (e.g., Mermaid or PlantUML)

### Integrated Test Engineering & Management

PREEvision links requirements directly to test cases and tracks test execution. req1 has no test artifact model.

- [ ] Test case entity (linked to requirements via `verifies` link type)
- [ ] Test execution entity (status, executor, timestamp, evidence)
- [ ] Test coverage metrics (% requirements with linked passing tests)
- [ ] Test status dashboard per module

### Reuse / Placeholder Mechanism

PREEvision allows requirements to be reused across modules as placeholders (embedded references that stay in sync with the source).

- [ ] Placeholder / proxy object type (references a source object in another module)
- [ ] Sync mechanism: placeholder reflects source object's current content
- [ ] UI: distinguish placeholders from native objects visually
- [ ] Break-link action to convert placeholder to independent copy

### Excel Import / Export

PREEvision supports Excel as a primary exchange format. Already on Priority 1 implicitly but elevated here.

- [ ] Excel export (XLSX with attribute columns, metadata sheet)
- [ ] Excel import (map columns to attributes, create/update objects)
- [ ] Round-trip: preserve object IDs across export → edit → reimport

### Deep Links from Published Reports

PREEvision generates reports with hyperlinks back into the model.

- [ ] Published HTML includes deep links to req1 UI for each object
- [ ] Anchor-based navigation (object ID in URL fragment)

### Review Voting & Approval Workflow

PREEvision has formal voting projects where stakeholders vote on requirements, with a dedicated chat view for discussion.

- [ ] Voting entity (vote per reviewer per artifact: approve / reject / abstain)
- [ ] Voting dashboard (aggregated results per review package)
- [ ] Chat/discussion view scoped to review package (not just per-object comments)
- [ ] Status transitions on review packages (draft → open → in_review → approved / rejected)

### Rich Text Editor with Tables & Graphics

PREEvision has a full rich text editor with embedded tables and graphics for requirement descriptions.

- [ ] TipTap rich text editor integration (already planned in solution strategy)
- [ ] Table support in requirement body (TipTap table extension)
- [ ] Image embedding in requirement body (upload + inline display)
- [ ] Graphics / diagram embedding (paste or drag-and-drop)

---

## Priority 6 — Polarion ALM Parity

Features identified from Siemens Polarion ALM that are missing or incomplete in req1. Items already covered by PREEvision Priority 5 are not repeated.

### LiveDoc / Document View (Dual Nature)

Polarion's core innovation: LiveDocs are Word-like documents where every paragraph is simultaneously a traceable, workflow-controlled database object. req1 is purely object-based with no document view.

- [ ] Document view entity (LiveDoc equivalent: ordered sequence of objects rendered as a continuous document)
- [ ] WYSIWYG document editing mode (paragraph = object, preserving traceability)
- [ ] Switch between document view and grid/object view for the same module
- [ ] Document outline / navigator sidebar in document mode
- [ ] Export document view to Word/PDF preserving formatting

### Electronic Signatures (E-Signatures)

Polarion supports FDA 21 CFR Part 11 compliant e-signatures: username + password confirmation on workflow transitions. Required for medical devices and pharma.

- [ ] E-signature mechanism (re-authentication on critical transitions)
- [ ] Signature audit record (user, timestamp, meaning, signature hash)
- [ ] E-signature requirement configurable per workflow transition
- [ ] Four-eyes principle enforcement (signer ≠ author)

### Word Import / Export (Round-Trip)

Polarion has a rule-based import wizard that maps Word paragraphs to requirements and supports export → edit → reimport.

- [ ] Word (.docx) export from module (structured document with styles)
- [ ] Word import wizard (map headings/paragraphs to object types and attributes)
- [ ] Round-trip: track object IDs through export → external edit → reimport

### Form Layout Designer

Polarion allows admins to design per-type Work Item forms: which fields appear, in what order, grouped into sections.

- [ ] Configurable object form layout per object type
- [ ] Form section grouping for attributes
- [ ] Form layout editor in admin UI

### Dependent Enumerations (Cascading Fields)

Polarion enum field values can depend on another enum field's selection (cascading dropdowns).

- [ ] Dependent enum definitions (parent enum → child enum value filtering)
- [ ] UI: cascading dropdown behavior in object editor

### Scheduled Scripts (CRON)

Polarion supports server-side scripts that run on configurable CRON schedules for automated validation, report generation, and cleanup.

- [ ] Script scheduling (CRON expression per script)
- [ ] Scheduled script execution engine (background job runner)
- [ ] Execution log with status, duration, output

### Save Hooks / Interceptors

Polarion has Java-based hooks that execute before/after saving a Work Item for custom validation, auto-population, and cross-artifact updates.

- [ ] Pre-save / post-save hook framework (extends JavaScript trigger system)
- [ ] Hook ordering and priority
- [ ] Hook failure blocks save with error message

### @Mentions in Comments

Polarion supports @mentioning users in comments with notification delivery.

- [ ] @mention syntax in comments (parse `@username`)
- [ ] User autocomplete dropdown on `@` input
- [ ] Notification delivery to mentioned users

### Real-Time Collaboration Awareness

Polarion shows visual indicators when multiple users edit the same document simultaneously.

- [ ] WebSocket-based presence awareness (who else is viewing/editing this module)
- [ ] Visual indicator for concurrent editors
- [ ] Conflict warning on concurrent saves

### Cross-Project Dashboards & Reporting

Polarion supports LiveReport pages with drag-and-drop widgets querying across multiple projects.

- [ ] Dashboard entity with configurable widget layout
- [ ] Cross-module/cross-project data aggregation in widgets
- [ ] Widget library (coverage chart, suspect link count, lifecycle distribution, test status)
- [ ] Dashboard export to PDF

### Compliance Project Templates

Polarion ships pre-configured project templates for regulated industries (IEC 62304, ISO 26262, DO-178C, Automotive SPICE).

- [ ] Project template entity (workspace template with pre-configured modules, attribute defs, object types, lifecycle models, scripts)
- [ ] Built-in templates for ISO 26262, DO-178C, IEC 62304
- [ ] Template instantiation wizard

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

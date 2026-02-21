# TODO — Gap Analysis vs Doorstop & DOORS Classic

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
- [x] Lua scripting — triggers (pre_save, pre_delete, validate), actions, layout stub
- [x] Module baselines with snapshot diffing (word-level inline diff for heading/body/attributes)
- [x] Validation service — built-in structural rules + Lua custom rules + required_attributes
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
- [x] Lua function returns a display value for a column
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

## Architecture Notes

### Lessons from Doorstop

1. **Zero infrastructure mode** — Doorstop works with just a git repo. Consider SQLite mode for local/offline use (`req1 init --local`).
2. **Git hooks for CI** — Doorstop ships pre-commit hooks. Our `req1 validate` already returns exit code 1 — document the CI integration pattern.
3. **One-file-per-item** — minimizes merge conflicts in distributed teams. Not applicable to our DB approach, but consider git-based export for offline workflows.
4. **Multiple publish formats are table-stakes** — Doorstop has 4 output formats. We need at least Markdown and LaTeX.

### Lessons from DOORS Classic

1. **Object type system** is the single biggest abstraction gap. Typed schemas > freeform attributes.
2. **Saved views** are essential for daily multi-user use. Different stakeholders need different views of the same module.
3. **Layout DXL / computed columns** — nobody else has this as an open-source feature. Completing our Lua layout scripts would be a differentiator.
4. **Link modules** — treating links as first-class entities enables separate access control and management.
5. **Impact analysis** is the killer feature of any traceability tool. We have the data, we need the visualization.

### Our Advantages Over Both

1. **Modern stack** — Rust (safety, perf), React, PostgreSQL vs Python/Tkinter or legacy C++/Windows-only.
2. **Server-based with full CRUD web UI** — Doorstop web is read-only. DOORS is desktop-only.
3. **Structured baseline diffing** — word-level inline diff with attribute comparison. Neither reference has this.
4. **Lua scripting** — more powerful than Doorstop's validator-only plugins, cleaner than DXL.
5. **Cross-module named typed links** — better traceability model than Doorstop's single unnamed parent-child links.

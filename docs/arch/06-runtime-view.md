# 6. Runtime View

## 6.1 OIDC Login Flow

```mermaid
sequenceDiagram
    actor User
    participant SPA as React SPA
    participant Nginx as nginx
    participant API as Axum API
    participant IdP as Corporate IdP
    participant Redis

    User->>SPA: Click "Login with SSO"
    SPA->>Nginx: GET /api/auth/oidc/login
    Nginx->>API: Proxy request
    API->>API: Generate PKCE challenge + state
    API->>Redis: Store state + PKCE verifier
    API-->>SPA: 302 Redirect to IdP authorize endpoint

    SPA->>IdP: Redirect to authorization endpoint
    User->>IdP: Authenticate (credentials / MFA)
    IdP-->>SPA: 302 Redirect to /api/auth/oidc/callback?code=...&state=...

    SPA->>Nginx: GET /api/auth/oidc/callback?code=...&state=...
    Nginx->>API: Proxy request
    API->>Redis: Validate state, retrieve PKCE verifier
    API->>IdP: POST /token (authorization code + PKCE verifier)
    IdP-->>API: ID token + access token
    API->>API: Validate ID token, extract user claims
    API->>API: Upsert user record (create on first login)
    API->>Redis: Create session
    API-->>SPA: Set session cookie, redirect to app
```

## 6.2 Create / Edit Requirement

```mermaid
sequenceDiagram
    actor User
    participant SPA as React SPA
    participant API as Axum API
    participant PG as PostgreSQL

    User->>SPA: Edit attribute in grid cell
    SPA->>API: PATCH /api/modules/{id}/objects/{id}<br/>{ "attributes": { "status": "approved" } }
    API->>API: Validate RBAC (user has editor role on module)
    API->>API: Validate attribute type and value

    API->>PG: BEGIN transaction
    API->>PG: UPDATE object SET attributes = ..., updated_at = now()
    API->>PG: INSERT INTO object_history<br/>(object_id, version, attribute_values,<br/>changed_by, changed_at, change_type='modify')
    API->>PG: UPDATE link SET suspect = true<br/>WHERE source_object_id = {id}
    Note over API,PG: All outgoing links from this object<br/>are flagged as suspect
    API->>PG: COMMIT

    API-->>SPA: 200 OK { updated object }
    SPA->>SPA: Update grid cell in-place
```

## 6.3 Create Link + Suspect Detection

```mermaid
sequenceDiagram
    actor User
    participant SPA as React SPA
    participant API as Axum API
    participant PG as PostgreSQL

    User->>SPA: Create link (source → target, type: "satisfies")
    SPA->>API: POST /api/links<br/>{ source_id, target_id, link_type: "satisfies",<br/>  attributes: { rationale: "..." } }
    API->>API: Validate RBAC
    API->>API: Validate link type exists, source/target exist
    API->>PG: INSERT INTO link (source_id, target_id, type_id,<br/>attributes, suspect=false)
    API-->>SPA: 201 Created { link }

    Note over User,PG: Later... source object is modified

    User->>SPA: Edit source object body
    SPA->>API: PATCH /api/modules/{id}/objects/{source_id}
    API->>PG: UPDATE object, INSERT history
    API->>PG: UPDATE link SET suspect = true<br/>WHERE source_object_id = {source_id}
    API-->>SPA: 200 OK

    SPA->>SPA: Link indicator changes to ⚠ suspect
    Note over User: User reviews the link,<br/>confirms it is still valid
    User->>SPA: Click "Clear suspect"
    SPA->>API: PATCH /api/links/{id} { suspect: false }
    API->>PG: UPDATE link SET suspect = false
    API-->>SPA: 200 OK
```

## 6.4 Create Baseline + Diff

```mermaid
sequenceDiagram
    actor User
    participant SPA as React SPA
    participant API as Axum API
    participant PG as PostgreSQL

    User->>SPA: Click "Create Baseline" on module
    SPA->>API: POST /api/modules/{id}/baselines<br/>{ name: "v2.1-release", description: "..." }
    API->>API: Validate RBAC (admin or editor)

    API->>PG: BEGIN transaction
    API->>PG: INSERT INTO baseline (name, module_id,<br/>created_by, created_at, locked=true)
    API->>PG: INSERT INTO baseline_entry (baseline_id, object_id, version)<br/>SELECT new_baseline_id, object_id, current_version<br/>FROM object WHERE module_id = {id}
    Note over API,PG: Snapshot: one entry per object<br/>pointing to its current version
    API->>PG: COMMIT

    API-->>SPA: 201 Created { baseline }

    Note over User,PG: Later... user wants to compare baselines

    User->>SPA: Select baseline A and baseline B for comparison
    SPA->>API: GET /api/modules/{id}/baselines/diff?a={a}&b={b}

    API->>PG: SELECT a.object_id, a.version AS v_a, b.version AS v_b,<br/>ha.attribute_values AS attrs_a,<br/>hb.attribute_values AS attrs_b<br/>FROM baseline_entry a<br/>FULL OUTER JOIN baseline_entry b<br/>ON a.object_id = b.object_id<br/>JOIN object_history ha ON ...<br/>JOIN object_history hb ON ...<br/>WHERE a.baseline_id={a} AND b.baseline_id={b}<br/>AND (a.version != b.version OR a.object_id IS NULL<br/>OR b.object_id IS NULL)

    API-->>SPA: 200 OK { added: [...], removed: [...],<br/>modified: [{ object_id, changed_attrs: [...] }] }
    SPA->>SPA: Render structured diff view
```

## 6.5 ReqIF Import

```mermaid
sequenceDiagram
    actor User
    participant SPA as React SPA
    participant API as Axum API
    participant ReqIF as ReqIF Module
    participant PG as PostgreSQL

    User->>SPA: Upload .reqif file
    SPA->>API: POST /api/import/reqif<br/>Content-Type: multipart/form-data

    API->>ReqIF: Parse XML (quick-xml + serde)
    ReqIF->>ReqIF: Deserialize ReqIF XML into Rust structs<br/>(REQ-IF → SPEC-TYPES → SPEC-OBJECTS<br/>→ SPEC-RELATIONS → SPECIFICATIONS)

    ReqIF->>ReqIF: Map ReqIF types to req1 attribute definitions
    ReqIF->>ReqIF: Map SPEC-OBJECTS to req1 objects
    ReqIF->>ReqIF: Map SPEC-RELATIONS to req1 links

    ReqIF->>PG: BEGIN transaction
    ReqIF->>PG: Upsert attribute definitions
    ReqIF->>PG: Bulk INSERT objects with attributes (JSONB)
    ReqIF->>PG: Bulk INSERT object_history entries
    ReqIF->>PG: Bulk INSERT links
    ReqIF->>PG: COMMIT

    API-->>SPA: 200 OK { imported: { objects: 1234,<br/>links: 567, attributes: 42 } }
    SPA->>SPA: Show import summary
```

## 6.6 Formal Review Workflow

```mermaid
sequenceDiagram
    actor Author
    actor Reviewer1
    actor Reviewer2
    participant SPA as React SPA
    participant API as Axum API
    participant PG as PostgreSQL

    Author->>SPA: Create review for module baseline
    SPA->>API: POST /api/reviews<br/>{ module_id, baseline_id,<br/>  participants: [reviewer1, reviewer2],<br/>  deadline: "2026-03-15" }
    API->>PG: INSERT INTO review, review_participant
    API-->>SPA: 201 Created

    Note over Reviewer1,Reviewer2: Reviewers are notified

    Reviewer1->>SPA: Open review, examine objects
    Reviewer1->>SPA: Approve object #42, reject object #87 with comment
    SPA->>API: POST /api/reviews/{id}/decisions<br/>{ object_id: 42, decision: "approved" }
    SPA->>API: POST /api/reviews/{id}/decisions<br/>{ object_id: 87, decision: "rejected",<br/>  comment: "Ambiguous wording in condition" }
    API->>PG: INSERT INTO review_decision (reviewer, object_id,<br/>decision, comment, timestamp)

    Reviewer2->>SPA: Approve all objects
    SPA->>API: POST /api/reviews/{id}/decisions (bulk)
    API->>PG: INSERT INTO review_decision (bulk)

    Author->>SPA: Check review status
    SPA->>API: GET /api/reviews/{id}/status
    API->>PG: Aggregate decisions per object
    API-->>SPA: { objects: [<br/>  { id: 42, approved: 2, rejected: 0 },<br/>  { id: 87, approved: 1, rejected: 1 } ] }
    SPA->>SPA: Render review dashboard with per-object status
```

## 6.7 OSLC Cross-Tool Link Creation

```mermaid
sequenceDiagram
    actor User
    participant SPA as React SPA
    participant API as Axum API
    participant OSLC as OSLC Module
    participant ExtTool as External Tool (Polarion/Jama/DOORS Next)
    participant PG as PostgreSQL

    User->>SPA: Click "Link to external tool" on object
    SPA->>API: GET /api/oslc/providers
    API->>PG: Fetch registered OSLC service providers
    API-->>SPA: [{ name: "Polarion", catalog_uri: "..." }, ...]

    User->>SPA: Select provider "Polarion"
    SPA->>API: GET /api/oslc/providers/{id}/selection-dialog
    API->>OSLC: Discover service provider catalog
    OSLC->>ExtTool: GET {catalog_uri}
    ExtTool-->>OSLC: Service provider document (JSON-LD)
    OSLC->>OSLC: Extract delegated selection dialog URI
    API-->>SPA: { dialog_uri: "https://polarion.corp/oslc/selector/..." }

    SPA->>SPA: Open delegated selection dialog in iframe/popup
    User->>ExtTool: Browse and select target requirement in Polarion
    ExtTool-->>SPA: postMessage({ oslc:results: [{ uri, title, ... }] })

    SPA->>API: POST /api/links<br/>{ source_object_id: "{req1_id}",<br/>  external_uri: "https://polarion.corp/oslc/rm/req/42",<br/>  link_type: "satisfies" }
    API->>PG: INSERT INTO link<br/>(source_object_id, external_uri, link_type_id, suspect=false)
    API-->>SPA: 201 Created { link with external_uri }

    Note over User,PG: Link now traceable across tools.<br/>Suspect detection applies when<br/>source object is modified.
```

## 6.8 Lua Script Execution

```mermaid
sequenceDiagram
    actor User
    participant SPA as React SPA
    participant API as Axum API
    participant Lua as Script Engine (mlua)
    participant PG as PostgreSQL

    User->>SPA: Open script editor, write Lua script
    Note over SPA: script = [[<br/>  local objects = req1.find_objects({<br/>    module_id = ctx.module_id,<br/>    filter = { status = "draft" }<br/>  })<br/>  for _, obj in ipairs(objects) do<br/>    req1.update_object(obj.id, {<br/>      attributes = { status = "in_review" }<br/>    })<br/>  end<br/>  return #objects .. " objects updated"<br/>]]

    SPA->>API: POST /api/scripts/execute<br/>{ body: "...", context: { module_id: "..." } }
    API->>API: Validate RBAC (user has editor role)
    API->>Lua: Create sandboxed Lua VM<br/>(memory limit: 64 MB, time limit: 30s)
    Lua->>Lua: Register req1.* API bindings
    Lua->>Lua: Execute script

    loop For each req1.find_objects / req1.update_object call
        Lua->>PG: Query/update via sea-orm
        PG-->>Lua: Results
    end

    Lua-->>API: Script result: "47 objects updated"
    API->>API: Log script execution in audit trail
    API-->>SPA: 200 OK { result: "47 objects updated",<br/>  duration_ms: 230, objects_read: 47, objects_written: 47 }
```

## 6.9 Roundtrip Export → External Edit → Reimport

```mermaid
sequenceDiagram
    actor User
    participant SPA as React SPA
    participant API as Axum API
    participant RT as Roundtrip Module
    participant PG as PostgreSQL
    actor External as External Editor (Excel/Word)

    User->>SPA: Click "Export for roundtrip"
    SPA->>API: POST /api/modules/{id}/roundtrip/export<br/>{ format: "json" }

    API->>RT: Build roundtrip package
    RT->>PG: SELECT objects with attributes from module
    PG-->>RT: Object data

    RT->>RT: For each object:<br/>- Compute content hash (SHA-256 of heading + body + attributes)<br/>- Include version, object_id, export_timestamp
    RT->>RT: Package as JSON with manifest

    API-->>SPA: 200 OK (download .req1.json)
    Note over SPA: Package structure:<br/>{ manifest: { module_id, exported_at, version_map },<br/>  objects: [{ id, version, hash, heading, body, attrs }] }

    User->>External: Open in Excel/Word, edit headings and bodies
    Note over External: User modifies REQ-042 heading,<br/>adds new object REQ-099,<br/>leaves others unchanged

    User->>SPA: Upload modified .req1.json
    SPA->>API: POST /api/modules/{id}/roundtrip/import<br/>Content-Type: multipart/form-data

    API->>RT: Parse reimport package
    RT->>PG: Fetch current objects for module
    RT->>RT: For each object in package:<br/>1. Compute hash of imported content<br/>2. Compare to original export hash<br/>3. Compare to current DB hash

    RT->>RT: Classify changes:<br/>- REQ-042: MODIFIED externally (hash mismatch)<br/>- REQ-099: ADDED (no original hash)<br/>- Others: UNCHANGED

    API-->>SPA: 200 OK { delta:<br/>  modified: [{ id: "REQ-042", field: "heading",<br/>    original: "...", imported: "...", current: "..." }],<br/>  added: [{ heading: "REQ-099", ... }],<br/>  conflicts: [] }

    SPA->>SPA: Render 3-way merge UI
    User->>SPA: Accept REQ-042 change, accept REQ-099 addition
    SPA->>API: POST /api/modules/{id}/roundtrip/apply<br/>{ accept: ["REQ-042", "REQ-099"], reject: [] }

    API->>PG: BEGIN transaction
    API->>PG: UPDATE objects, INSERT history, INSERT new objects
    API->>PG: COMMIT

    API-->>SPA: 200 OK { applied: 2, rejected: 0 }
```

## 6.10 MCP Tool Invocation

```mermaid
sequenceDiagram
    actor AI as AI Assistant (Claude)
    participant MCP as MCP Server (req1)
    participant API as Axum API
    participant PG as PostgreSQL

    AI->>MCP: tools/call { name: "search_requirements",<br/>  arguments: { query: "performance latency",<br/>    module_id: "..." } }

    MCP->>API: Internal query (FTS + module filter)
    API->>PG: SELECT * FROM object<br/>WHERE module_id = $1<br/>AND to_tsvector(heading || body) @@ plainto_tsquery($2)
    PG-->>API: Matching objects

    MCP-->>AI: { content: [{ type: "text",<br/>  text: "Found 3 requirements:\n1. REQ-042: ..." }] }

    AI->>MCP: tools/call { name: "check_requirement_quality",<br/>  arguments: { object_id: "REQ-042" } }

    MCP->>API: Fetch object
    API->>PG: SELECT * FROM object WHERE id = $1
    PG-->>API: Object data
    MCP->>MCP: Apply quality rules:<br/>- INCOSE: ambiguity, passive voice, vague terms<br/>- EARS: pattern compliance<br/>- Completeness: missing attributes

    MCP-->>AI: { content: [{ type: "text",<br/>  text: "Quality issues found:\n- Passive voice: 'shall be handled'\n- Vague term: 'quickly'" }] }
```

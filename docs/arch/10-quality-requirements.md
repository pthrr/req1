# 10. Quality Requirements

## 10.1 Quality Tree

```
Quality
├── Performance
│   ├── Grid rendering latency
│   ├── API response time
│   └── Import/export throughput
├── Security
│   ├── Authentication (OIDC + local)
│   ├── Authorization (RBAC)
│   ├── Data protection (encryption, headers)
│   └── Input validation
├── Usability
│   ├── Grid editing efficiency
│   ├── Navigation and filtering
│   └── Keyboard-driven workflows
├── Interoperability
│   ├── ReqIF round-trip fidelity
│   ├── OSLC cross-tool traceability
│   ├── SysML v2 requirement interchange
│   ├── Roundtrip export delta fidelity
│   ├── REST API completeness
│   ├── Lua scripting API coverage
│   └── CLI automation
├── Auditability
│   ├── Mutation history
│   ├── Baseline immutability
│   └── Compliance reporting
└── Availability
    ├── Uptime target
    ├── Graceful degradation
    └── Backup and recovery
```

## 10.2 Quality Scenarios

### Performance

| ID | Scenario | Metric |
|----|----------|--------|
| QS-P1 | Engineer opens a module with 10,000 objects in grid view | Grid renders and is interactive within **2 seconds** |
| QS-P2 | Engineer applies a filter on a 10,000-object module | Filtered results display within **500 ms** |
| QS-P3 | Engineer saves an attribute edit on a single object | API response within **200 ms** (including history write) |
| QS-P4 | Admin imports a 5,000-object ReqIF file | Import completes within **30 seconds** |
| QS-P5 | Engineer requests a baseline diff (two baselines, 10,000 objects) | Diff result returned within **3 seconds** |

### Security

| ID | Scenario | Metric |
|----|----------|--------|
| QS-S1 | Unauthenticated user accesses any API endpoint | Returns **401 Unauthorized** — no data leakage |
| QS-S2 | Viewer-role user attempts to edit an object | Returns **403 Forbidden** |
| QS-S3 | User in Program A attempts to access Program B data | Returns **404 Not Found** (program isolation) |
| QS-S4 | Malicious XHTML injected in object body | Sanitized before storage — no stored XSS |
| QS-S5 | All API responses include security headers | CSP, HSTS, X-Frame-Options, X-Content-Type-Options present |

### Usability

| ID | Scenario | Metric |
|----|----------|--------|
| QS-U1 | New user (familiar with DOORS) performs basic CRUD | Productive within **30 minutes** without training |
| QS-U2 | Engineer edits 50 objects in grid view | Inline editing, tab navigation, no page reloads — **spreadsheet-like** experience |
| QS-U3 | Engineer creates a traceability link between modules | Completed in **3 clicks or fewer** |

### Interoperability

| ID | Scenario | Metric |
|----|----------|--------|
| QS-I1 | ReqIF file exported from DOORS Classic is imported into req1 | **All** attributes, objects, and links preserved |
| QS-I2 | Objects exported from req1 as ReqIF, re-imported into DOORS Classic | **Round-trip** preserves all standard attribute types |
| QS-I3 | CI/CD pipeline queries objects via REST API | **Every** UI operation is available via API |
| QS-I4 | CLI script creates a baseline and exports a report | CLI covers baseline creation, querying, and export |
| QS-I5 | External tool (Polarion) links to a req1 requirement via OSLC | Delegated selection dialog opens, user selects object, link stored with external URI |
| QS-I6 | req1 creates a link to a Jama requirement via OSLC | OSLC consumer discovers provider catalog, opens selection dialog, link stored |
| QS-I7 | SysML v2 modeling tool imports req1 requirements | All `RequirementUsage` elements exported with attributes preserved |
| QS-I8 | SysML v2 model requirements imported into req1 | `RequirementUsage` mapped to req1 objects, `SatisfyRequirementUsage` mapped to links |
| QS-I9 | Module exported for roundtrip, edited externally, reimported | Per-object deltas detected correctly. Modified objects identified. No false positives from unchanged objects. |
| QS-I10 | Lua script batch-updates 500 objects in a module | Script completes within **10 seconds**, all mutations recorded in audit trail |
| QS-I11 | AI assistant queries "show all untested ASIL-D requirements" via MCP | MCP tool returns correct results, respecting user's RBAC scope |

### Auditability

| ID | Scenario | Metric |
|----|----------|--------|
| QS-A1 | Auditor queries "who changed attribute X on object Y" | Exact answer available from `object_history` — **attribute-level granularity** |
| QS-A2 | Every mutation (create, modify, delete) on any object | Recorded in history table with user, timestamp, and change type — **zero exceptions** |
| QS-A3 | Baseline created and later queried | Baseline is **immutable** — entries cannot be modified after creation |
| QS-A4 | Compliance audit for ISO 26262 project | Complete traceability chain from hazard → safety goal → requirement → test case is queryable |
| QS-A5 | Compliance audit for DO-178C project | Bidirectional traceability between high-level requirements, low-level requirements, source code references, and test cases |
| QS-A6 | Risk matrix review for IEC 61508 project | All hazards visible in risk matrix with linked mitigating requirements and their SIL assignments |
| QS-A7 | Lua script modifies 100 objects | All 100 mutations attributed to the executing user in `object_history` with `change_type = 'script'` |

### Availability

| ID | Scenario | Metric |
|----|----------|--------|
| QS-AV1 | Production Kubernetes deployment | **99.9% uptime** (< 8.8 hours downtime/year) |
| QS-AV2 | Single Axum replica fails | Load balancer routes to remaining replicas — **zero user-visible impact** |
| QS-AV3 | PostgreSQL primary fails | Replica promoted — recovery within **5 minutes** (RPO: 0 with synchronous replication) |
| QS-AV4 | Redis unavailable | Axum falls back to stateless mode — degraded but **functional** |

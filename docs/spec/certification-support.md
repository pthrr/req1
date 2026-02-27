# Certification Support Requirements

## Scope

req1 is a **requirements management tool** (Scope A). It owns the integrity of requirement data and produces auditable evidence within the requirements domain. Cross-tool lifecycle traceability (requirement → design → code → test across tools) is delegated to external traceability tools (e.g., Reqtify, Reqwise) via the integration interfaces defined in [functional.md §11](functional.md#11-integration-interfaces--fr-11xx).

This document specifies what req1 must guarantee so that:

1. Its output is **trustworthy** — auditors can rely on exported data.
2. It does not **introduce errors** — no silent data corruption, no lost links.
3. It does not **fail to detect problems** — suspect links, orphans, and coverage gaps are surfaced.
4. It produces **reproducible evidence** — the same query on the same baseline yields the same result.

These properties are required for tool qualification under DO-330 (Qualification of Tools, companion to DO-178C), ISO 26262 Part 8 (Supporting Processes), and IEC 62304 §5.1 (Software Development Planning — tool validation).

---

## 1. Data Integrity — CERT-1xx

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| CERT-100 | The system SHALL NOT silently drop, corrupt, or alter requirement data during any operation (create, update, delete, import, export, baseline). | SHALL | Implemented |
| CERT-101 | All database writes SHALL use transactions. A failed transaction SHALL NOT leave the database in an inconsistent state. | SHALL | Implemented |
| CERT-102 | Object content fingerprints (SHA-256) SHALL be recomputed and verified on every mutation. A mismatch between stored fingerprint and computed fingerprint SHALL raise an error. | SHALL | Implemented |
| CERT-103 | Baseline entries SHALL be immutable after creation. The system SHALL NOT provide any API to modify or delete individual baseline entries. | SHALL | Implemented |
| CERT-104 | ReqIF export SHALL produce output that, when reimported into req1, yields identical objects, attributes, and links (round-trip fidelity). | SHALL | Planned |
| CERT-105 | CSV and Excel export SHALL include all attributes and metadata necessary to uniquely identify and reconstruct each object. | SHALL | Planned |
| CERT-106 | The system SHALL validate all user input at the API boundary. Invalid input SHALL be rejected with a descriptive error before any database write. | SHALL | Implemented |
| CERT-107 | Rich text (XHTML) content SHALL be sanitized before storage to prevent stored XSS. Sanitization SHALL NOT alter the semantic content of the requirement text. | SHALL | Implemented |
| CERT-108 | Database migrations SHALL be forward-only and non-destructive. Migrations SHALL NOT delete or alter existing data except through explicitly documented schema evolution. | SHALL | Implemented |

---

## 2. Traceability Evidence — CERT-2xx

These requirements apply to traceability **within the requirements domain** (requirement-to-requirement). Cross-tool traceability (requirement → design → code → test) is the responsibility of external tools consuming req1 data via the integration interfaces (FR-11xx).

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| CERT-200 | The system SHALL maintain directed, typed links between objects. Each link SHALL record: source object, target object, link type, creation timestamp, and creating user. | SHALL | Implemented |
| CERT-201 | The system SHALL detect and flag suspect links automatically when a linked source object is modified. | SHALL | Implemented |
| CERT-202 | The traceability matrix SHALL display all links between two modules without omission. The matrix SHALL indicate suspect status for each link. | SHALL | Implemented |
| CERT-203 | Coverage metrics SHALL accurately report the percentage of objects with upstream and downstream links. An object with zero links in a given direction SHALL be counted as uncovered. | SHALL | Implemented |
| CERT-204 | Impact analysis SHALL traverse all reachable objects via link chains up to the configured depth. No reachable object SHALL be omitted from the result. | SHALL | Implemented |
| CERT-205 | The system SHALL support identifying orphan objects (objects with no links in any direction) within a module. | SHALL | Implemented |
| CERT-206 | All traceability queries (matrix, coverage, impact) SHALL operate on the current live data or on a specified baseline. Baseline-based queries SHALL use the frozen snapshot, not current data. | SHALL | Partial |
| CERT-207 | Traceability data SHALL be exportable via ReqIF (FR-800) and REST API (FR-1100) in formats consumable by external traceability tools. | SHALL | Partial |

---

## 3. Change Management — CERT-3xx

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| CERT-300 | Every object modification SHALL increment the object's version counter. | SHALL | Implemented |
| CERT-301 | The system SHALL support creating immutable baselines that capture the exact version of every object in a module at the time of creation. | SHALL | Implemented |
| CERT-302 | The system SHALL support structured comparison (diff) between any two baselines of the same module, classifying each object as added, removed, modified, or unchanged. | SHALL | Implemented |
| CERT-303 | For modified objects, the diff SHALL provide field-level and word-level change detail sufficient to identify exactly what changed. | SHALL | Implemented |
| CERT-304 | Soft-deleted objects SHALL remain in the database and be recoverable. The system SHALL NOT permanently destroy data without explicit, separate action. | SHALL | Implemented |
| CERT-305 | Baseline sets SHALL enable grouping baselines across modules to represent a consistent system-wide configuration state. | SHALL | Partial |
| CERT-306 | The system SHALL support change proposals that record proposed modifications with diff data before application. | SHALL | Partial |

---

## 4. Audit Trail — CERT-4xx

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| CERT-400 | Every mutation (create, modify, delete) on any object SHALL be recorded in the object history table. | SHALL | Implemented |
| CERT-401 | Each history entry SHALL record: object ID, module ID, version, full attribute snapshot (JSONB), changed_by (user ID), changed_at (timestamp), and change_type. | SHALL | Implemented |
| CERT-402 | History entries SHALL be append-only. The system SHALL NOT provide any API to modify or delete history entries. | SHALL | Implemented |
| CERT-403 | The system SHALL support querying history to answer: "who changed attribute X on object Y between dates A and B". | SHALL | Implemented |
| CERT-404 | All data mutations made by scripts SHALL be attributed to the executing user in the audit trail with `change_type = 'script'`. | SHALL | Implemented |
| CERT-405 | When authentication is implemented (FR-610), the `changed_by` field SHALL be populated from the authenticated user identity. | SHALL | Planned |
| CERT-406 | E-signature records (FR-630) SHALL be immutable and queryable: "who signed what, when, and with what meaning". | SHALL | Planned |
| CERT-407 | Lifecycle state transitions SHALL be recorded in the audit trail with `change_type = 'state_transition'`. | SHALL | Planned |
| CERT-408 | The system SHOULD support configurable history policy per module: `always` (default), `on_baseline`, or `off`. | SHOULD | Planned |

---

## 5. Standard Compliance Mapping — CERT-5xx

### DO-178C / DO-330 (Avionics)

req1 is classified as a **Criteria 3 tool** (output is used without verification by the user) under DO-330 Table T-0. Tool qualification requires demonstrating that the tool's output is correct.

| ID | Requirement | Standard Reference | How req1 Satisfies |
|----|-------------|-------------------|---------------------|
| CERT-500 | Bidirectional traceability between requirement levels SHALL be queryable. | DO-178C §5.5, Table A-3 objective 7 | Directed typed links (FR-310) + traceability matrix (FR-330) + impact analysis (FR-340). |
| CERT-501 | Requirements baselines SHALL be immutable and reproducible. | DO-178C §7.2.1 | Immutable baselines (FR-420, CERT-103) with structured diff (FR-430). |
| CERT-502 | Change history SHALL provide full attribution (who, what, when). | DO-178C §7.2.2, DO-330 §6.3.1 | Object history table (CERT-400, CERT-401) with append-only guarantees (CERT-402). |
| CERT-503 | Requirements data SHALL be exportable for independent verification. | DO-330 §6.3.4 | ReqIF export (FR-802), CSV export (FR-810), REST API (FR-1100). |
| CERT-504 | Suspect links SHALL be flagged when source requirements change. | DO-178C §5.5 | Auto-suspect detection (FR-320) on content fingerprint change (FR-410). |

### ISO 26262 (Automotive)

| ID | Requirement | Standard Reference | How req1 Satisfies |
|----|-------------|-------------------|---------------------|
| CERT-510 | The tool SHALL support traceability from safety goals through functional safety requirements to technical safety requirements. | ISO 26262-8 §11.4.5 | Cross-module links with named types (FR-310), traceability matrix (FR-330). |
| CERT-511 | The tool SHALL support impact analysis when requirements change. | ISO 26262-8 §8.4.3 | BFS impact traversal (FR-340) with configurable direction and depth. |
| CERT-512 | The tool SHALL support baseline comparison for change management. | ISO 26262-8 §8.4.4 | Baseline diff with word-level granularity (FR-430). |
| CERT-513 | The tool SHALL maintain a complete audit trail of all requirement changes. | ISO 26262-8 §11.4.2 | Object history (CERT-400) with append-only immutability (CERT-402). |
| CERT-514 | The tool SHALL support formal review workflows with traceable approval. | ISO 26262-8 §11.4.7 | Review packages (FR-900), voting (FR-920), e-signatures (FR-630). |
| CERT-515 | Tool qualification evidence SHALL be producible per ISO 26262-8 §11.4.9. | ISO 26262-8 §11.4.9 | Data export (CERT-503) + validation rules (FR built-in + JavaScript) + this specification. |

### IEC 62304 (Medical Devices)

| ID | Requirement | Standard Reference | How req1 Satisfies |
|----|-------------|-------------------|---------------------|
| CERT-520 | The tool SHALL support traceability between software requirements and higher-level system requirements. | IEC 62304 §5.1.1, §5.3.6 | Cross-module directed links (FR-310), coverage metrics (FR-350). |
| CERT-521 | The tool SHALL support change control with documented rationale. | IEC 62304 §6.1, §6.2 | Change proposals (FR-940) with diff data, audit trail (CERT-400). |
| CERT-522 | The tool SHALL support electronic signatures compliant with FDA 21 CFR Part 11. | FDA 21 CFR Part 11 §11.50, §11.70 | E-signatures (FR-630) with immutable records (CERT-406), four-eyes principle (FR-632). |
| CERT-523 | The tool SHALL support risk-based classification of requirements (software safety class). | IEC 62304 §4.3 | Typed attributes (FR-200) with integrity level field, object types (FR-210). |

### General Tool Qualification

| ID | Requirement | Priority | Status |
|----|-------------|----------|--------|
| CERT-530 | The system SHALL include built-in validation rules detecting structural issues (orphan objects, empty headings, missing required attributes, broken link references). | SHALL | Implemented |
| CERT-531 | The system SHALL support user-defined custom validation rules via JavaScript scripts (FR-710, validate trigger). | SHALL | Implemented |
| CERT-532 | The validation endpoint SHALL return a structured report with issue severity (error, warning, info), affected object, and description. | SHALL | Implemented |
| CERT-533 | The CLI `validate` command SHALL exit with code 1 when errors are found, enabling integration into CI/CD gating pipelines. | SHALL | Implemented |
| CERT-534 | The system SHALL provide a comprehensive REST API (FR-1100) enabling automated tool qualification test suites to verify all certification-relevant behaviors. | SHALL | Implemented |
| CERT-535 | The system SHOULD include integration tests that verify the correctness of traceability queries, baseline operations, and audit trail entries. | SHOULD | Implemented |

---

## Requirement Count Summary

| Status | Count |
|--------|-------|
| Implemented | 29 |
| Partial | 4 |
| Planned | 6 |
| **Total** | **39** |

## Combined Specification Summary

| Document | Implemented | Partial | Planned | Total |
|----------|-------------|---------|---------|-------|
| [functional.md](functional.md) | 109 | 4 | 63 | 176 |
| [certification-support.md](certification-support.md) | 29 | 4 | 6 | 39 |
| **Total** | **138** | **8** | **69** | **215** |

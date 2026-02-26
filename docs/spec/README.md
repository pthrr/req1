# req1 — Software Requirements Specification

## Purpose

This specification defines the functional requirements and certification support properties for **req1**, an open-source requirements management tool. It describes *what* the system must do. For *how* it is built, see [docs/arch/](../arch/00-table-of-contents.md) (arc42).

## Scope

req1 is a **requirements management tool** (Scope A). It owns:

- Authoring, structuring, and versioning of requirements
- Intra-tool traceability (requirement-to-requirement links, coverage, impact analysis)
- Baselines, change history, and audit trail
- Export of requirement data in standard formats

req1 does **not** own cross-tool lifecycle traceability (requirement → design → code → test across tools). That responsibility is delegated to external traceability tools (e.g., Reqtify, Reqwise) via the integration interfaces defined in this specification (ReqIF, OSLC, REST API).

## Conventions

### RFC 2119 Keywords

The keywords "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", and "MAY" in this document are to be interpreted as described in [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119).

| Keyword | Meaning |
|---------|---------|
| **SHALL** | Absolute requirement. The system must satisfy this. |
| **SHALL NOT** | Absolute prohibition. The system must not do this. |
| **SHOULD** | Recommended. May be omitted with documented justification. |
| **SHOULD NOT** | Discouraged. May be done with documented justification. |
| **MAY** | Optional. Implementation is at the discretion of the developer. |

### Requirement ID Scheme

| Prefix | Domain |
|--------|--------|
| FR-1xx | Data Model (workspaces, projects, modules, objects) |
| FR-2xx | Attributes & Object Types |
| FR-3xx | Links & Traceability |
| FR-4xx | Baselines & Versioning |
| FR-5xx | Views, Search & Filtering |
| FR-6xx | Access Control & Authentication |
| FR-7xx | Scripting Engine |
| FR-8xx | Import / Export & Interchange |
| FR-9xx | Review, Collaboration & Workflow |
| FR-10xx | Publishing & Reporting |
| FR-11xx | Integration Interfaces |
| CERT-1xx | Data Integrity |
| CERT-2xx | Traceability Evidence |
| CERT-3xx | Change Management |
| CERT-4xx | Audit Trail |
| CERT-5xx | Standard Compliance Mapping |

### Requirement Status

| Status | Meaning |
|--------|---------|
| **Implemented** | Requirement is satisfied in the current codebase. |
| **Partial** | Data model or backend exists, but UI or full workflow is incomplete. |
| **Planned** | Requirement is accepted and scheduled for a future phase. |

## Cross-References to arc42

The following concerns are defined in the architecture documentation and are **not** repeated here:

| Concern | Canonical Location |
|---------|--------------------|
| Stakeholders, scope, goals | [arch/01](../arch/01-introduction-and-goals.md) |
| Technical & organizational constraints | [arch/02](../arch/02-constraints.md) |
| System context & external interfaces | [arch/03](../arch/03-context-and-scope.md) |
| Solution strategy & tech stack | [arch/04](../arch/04-solution-strategy.md) |
| Component decomposition | [arch/05](../arch/05-building-block-view.md) |
| Domain model & data model | [arch/08](../arch/08-crosscutting-concepts.md) |
| Quality attributes & scenarios | [arch/10](../arch/10-quality-requirements.md) |
| Risks & technical debt | [arch/11](../arch/11-risks-and-technical-debt.md) |
| Glossary | [arch/12](../arch/12-glossary.md) |

## Documents

| File | Contents |
|------|----------|
| [functional.md](functional.md) | Functional requirements (FR-xxx) |
| [certification-support.md](certification-support.md) | Certification support requirements (CERT-xxx) |

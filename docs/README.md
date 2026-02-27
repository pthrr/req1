# Documentation

## Specification (SRS)

Software requirements specification using RFC 2119 language. Defines *what* the system must do.

| Document | Description |
|----------|-------------|
| [spec/README.md](spec/README.md) | Overview, scope, conventions, cross-references to arc42 |
| [spec/functional.md](spec/functional.md) | 196 functional requirements (FR-xxx) across 12 domains |
| [spec/certification-support.md](spec/certification-support.md) | 39 certification support requirements (CERT-xxx) for DO-178C, ISO 26262, IEC 62304 |

## Architecture (arc42)

Architecture documentation follows the [arc42](https://arc42.org) template. Defines *how* the system is built.

| # | Section | Description |
|---|---------|-------------|
| [01](arch/01-introduction-and-goals.md) | Introduction and Goals | Purpose, stakeholders, quality goals, MVP scope |
| [02](arch/02-constraints.md) | Constraints | Technical, organizational, and convention constraints |
| [03](arch/03-context-and-scope.md) | Context and Scope | System context diagram, business & technical context |
| [04](arch/04-solution-strategy.md) | Solution Strategy | Tech stack, key approaches, rejected alternatives |
| [05](arch/05-building-block-view.md) | Building Block View | Container and component diagrams |
| [06](arch/06-runtime-view.md) | Runtime View | Sequence diagrams for key workflows |
| [07](arch/07-deployment-view.md) | Deployment View | Kubernetes and Docker Compose deployments, devcontainer |
| [08](arch/08-crosscutting-concepts.md) | Crosscutting Concepts | Auth, RBAC, audit, versioning, search, export, observability |
| [09](arch/09-architecture-decisions.md) | Architecture Decisions | ADRs for all major technology choices |
| [10](arch/10-quality-requirements.md) | Quality Requirements | Quality tree and concrete scenarios |
| [11](arch/11-risks-and-technical-debt.md) | Risks and Technical Debt | Known risks and deferred features |
| [12](arch/12-glossary.md) | Glossary | Domain terms mapped from DOORS Classic to req1 |

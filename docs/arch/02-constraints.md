# 2. Constraints

## 2.1 Technical Constraints

| Constraint | Rationale |
|------------|-----------|
| **Rust** backend | Performance, memory safety, single-binary deployment. Async runtime via tokio. |
| **React + TypeScript** frontend (SPA) | AG Grid official bindings, React Flow, TipTap, react-arborist — ecosystem required for data-heavy UI. |
| **PostgreSQL** | Single source of truth for data, history, baselines, links. JSONB for flexible attributes. Built-in FTS. |
| **Self-hostable** | Enterprise customers require on-premise deployment. No mandatory cloud dependency. |
| **MIT license** | Most permissive open-source license. Chosen explicitly for maximum adoption. |
| **No Node.js in production** | Frontend compiled to static assets by Vite, served from the Axum binary via `rust-embed` or `tower-http::ServeDir`. |

## 2.2 Organizational Constraints

| Constraint | Rationale |
|------------|-----------|
| **DOORS Classic interop via ReqIF** | Existing customers must migrate from DOORS Classic. ReqIF 1.2 is the only standard interchange format. |
| **OSLC interop** | Cross-tool traceability in regulated environments requires OSLC Core 3.0 / OSLC-RM 2.1. Common in automotive and aerospace toolchains (Polarion, Jama, DOORS Next, Jira). |
| **SysML v2 interop** | Model-based systems engineering (MBSE) workflows require importing/exporting requirements from/to SysML v2 modeling tools. JSON/REST-based standard (OMG). |
| **Enterprise deployment readiness** | Must support OIDC SSO (Azure AD, Okta, Keycloak), RBAC, audit trails, and data classification from day one. |
| **Regulated industry compliance** | Users operate under ISO 26262, DO-178C, DO-254, IEC 62304, IEC 61508, EN 50128/50129, IEC 61511, ECSS-Q-ST-80C, IEC 62443 — every change must be traceable and auditable. |

## 2.3 Conventions

| Convention | Detail |
|------------|--------|
| **REST / OpenAPI** | All API endpoints specified via OpenAPI. API-first design — UI is a consumer, not the only consumer. |
| **Containerized deployment** | OCI images for all services. Kubernetes for production, Docker Compose for small teams. Devcontainer for onboarding. |
| **12-factor app** | Configuration via environment variables. Stateless processes. Backing services as attached resources. |
| **arc42 + C4** | Architecture documented using arc42 template sections with C4 model diagrams (Mermaid). |
| **ADR format** | Major decisions recorded as Architecture Decision Records (status, context, decision, consequences). |

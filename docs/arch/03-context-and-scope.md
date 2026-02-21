# 3. Context and Scope

## 3.1 System Context Diagram

```mermaid
C4Context
    title System Context — req1

    Person(engineer, "Systems Engineer", "Authors, traces, and baselines requirements")
    Person(admin, "Admin", "Manages users, roles, projects, and OIDC configuration")
    Person(reviewer, "Reviewer", "Reviews and approves requirements in formal review workflows")

    System(req1, "req1", "Open-source requirements management system replacing DOORS Classic")

    System_Ext(doors, "DOORS Classic", "Legacy RM tool — data exchanged via ReqIF 1.2 files")
    System_Ext(idp, "Corporate IdP", "OIDC provider (Azure AD, Okta, Keycloak) for SSO authentication")
    System_Ext(cicd, "CI/CD Pipeline", "Build systems consuming req1 REST API and webhooks")
    System_Ext(chat, "Slack / Teams", "Chat clients rendering link previews via OpenGraph meta tags")
    System_Ext(oslc_tools, "OSLC-Capable Tools", "Polarion, Jama, DOORS Next, Jira — cross-tool traceability via OSLC")
    System_Ext(sysml_tools, "SysML v2 Tools", "Cameo, Capella, SysML v2 API providers — model-based SE")
    System_Ext(ai_assistants, "AI Assistants", "Claude, other MCP clients — AI-assisted quality checks and queries")

    Rel(engineer, req1, "Authors and traces requirements", "HTTPS")
    Rel(admin, req1, "Configures projects, users, roles", "HTTPS")
    Rel(reviewer, req1, "Reviews and approves objects", "HTTPS")

    Rel(req1, doors, "Imports/exports requirements", "ReqIF 1.2 XML files")
    Rel(req1, idp, "Authenticates users", "OIDC / OAuth 2.0")
    Rel(cicd, req1, "Queries and updates requirements", "REST API / Webhooks")
    Rel(chat, req1, "Fetches link previews", "HTTP GET with OG tags")
    Rel(req1, oslc_tools, "Cross-tool traceability links", "OSLC Core 3.0 / JSON-LD")
    Rel(oslc_tools, req1, "Discovers and links to req1 objects", "OSLC RM 2.1")
    Rel(req1, sysml_tools, "Exports requirement elements", "SysML v2 JSON API")
    Rel(sysml_tools, req1, "Imports requirement elements", "SysML v2 JSON API")
    Rel(ai_assistants, req1, "Queries and analyzes requirements", "MCP / JSON-RPC")
```

## 3.2 Business Context

| Partner | Input | Output | Protocol |
|---------|-------|--------|----------|
| Systems Engineer | Requirement edits, link definitions, baseline requests | Updated objects, traceability reports, baseline diffs | HTTPS (SPA) |
| Admin | User/role/project configuration | Confirmation, audit entries | HTTPS (SPA) |
| Reviewer | Review decisions (approve/reject/comment) | Review status, notifications | HTTPS (SPA) |
| DOORS Classic | ReqIF 1.2 export files | ReqIF 1.2 import files | File exchange |
| Corporate IdP | OIDC discovery, token exchange | ID token, user claims | OIDC / OAuth 2.0 |
| CI/CD Pipeline | REST API calls, webhook subscriptions | JSON responses, webhook events | HTTPS |
| Slack / Teams | HTTP GET for shared links | HTML with OpenGraph meta tags | HTTPS |
| OSLC-Capable Tools | OSLC resource discovery, selection dialogs, link creation | OSLC RM resources (JSON-LD/RDF), service provider catalog | OSLC Core 3.0 / HTTPS |
| SysML v2 Tools | SysML v2 requirement elements (JSON) | SysML v2 requirement elements (JSON) | SysML v2 REST API / HTTPS |
| AI Assistants | MCP tool calls (search, analyze, quality check) | Structured results (objects, coverage, quality reports) | MCP / JSON-RPC / HTTPS |

## 3.3 Technical Context

| Channel | Protocol | Format | Notes |
|---------|----------|--------|-------|
| Browser → nginx → Axum | HTTPS | JSON (REST API), HTML/JS/CSS (SPA) | TLS terminated at nginx |
| Axum → PostgreSQL | TCP | SQL (sea-orm / sqlx) | Connection pool via deadpool/sqlx |
| Axum → Redis | TCP | RESP | Sessions, cache, rate limiting |
| Axum → SeaweedFS | HTTP | S3-compatible API | File attachments, images |
| Axum → Corporate IdP | HTTPS | OIDC (JSON) | Token exchange, user info |
| Axum → Webhook targets | HTTPS | JSON | Event notifications to CI/CD |
| CLI → Axum | HTTPS | JSON (REST API) | Headless automation |
| Axum → OSLC Tools | HTTPS | JSON-LD (OSLC Core 3.0) | Cross-tool link creation (client role) |
| OSLC Tools → Axum | HTTPS | JSON-LD (OSLC RM 2.1) | Resource discovery and selection (server role) |
| Axum ↔ SysML v2 Tools | HTTPS | JSON (SysML v2 API) | Requirement element import/export |
| MCP Client → Axum | HTTPS | JSON-RPC (MCP) | AI assistant tool invocation |

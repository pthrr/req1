import { useState } from "react";
import { Outlet, useLocation, useParams } from "react-router";
import { Sidebar } from "./Sidebar";
import { theme } from "./theme";

export function App() {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100vh", fontFamily: theme.fontFamily }}>
      {/* Header */}
      <header
        style={{
          height: theme.header.height,
          borderBottom: `1px solid ${theme.colors.headerBorder}`,
          background: theme.colors.headerBg,
          display: "flex",
          alignItems: "center",
          padding: `0 ${theme.spacing.lg}`,
          flexShrink: 0,
        }}
      >
        <Breadcrumbs />
      </header>

      {/* Body: sidebar + main */}
      <div style={{ display: "flex", flex: 1, overflow: "hidden" }}>
        <Sidebar
          collapsed={sidebarCollapsed}
          onToggleCollapse={() => setSidebarCollapsed((p) => !p)}
        />
        <main style={{ flex: 1, overflow: "auto", padding: theme.spacing.lg }}>
          <Outlet />
        </main>
      </div>
    </div>
  );
}

function Breadcrumbs() {
  const location = useLocation();
  const params = useParams();
  const parts: string[] = ["req1"];

  // Build breadcrumbs from URL params
  if (params.workspaceId) {
    parts.push(params.workspaceId.slice(0, 8) + "...");
  }
  if (params.projectId) {
    parts.push(params.projectId.slice(0, 8) + "...");
  }
  if (params.moduleId) {
    parts.push(params.moduleId.slice(0, 8) + "...");
  }
  if (location.pathname.endsWith("/traceability")) {
    parts.push("Traceability Matrix");
  }

  return (
    <span style={{ fontSize: "0.9rem", color: theme.colors.text }}>
      {parts.map((p, i) => (
        <span key={i}>
          {i > 0 && <span style={{ margin: `0 ${theme.spacing.xs}`, color: theme.colors.textMuted }}>&gt;</span>}
          <span style={{ fontWeight: i === parts.length - 1 ? 600 : 400 }}>{p}</span>
        </span>
      ))}
    </span>
  );
}

import { useEffect, useState } from "react";
import { useNavigate, useParams } from "react-router";
import {
  api,
  type Module,
  type Project,
  type Workspace,
} from "./api/client";
import { theme } from "./theme";

interface Props {
  collapsed: boolean;
  onToggleCollapse: () => void;
}

export function Sidebar({ collapsed, onToggleCollapse }: Props) {
  const navigate = useNavigate();
  const params = useParams();

  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [expandedWorkspaces, setExpandedWorkspaces] = useState<Set<string>>(new Set());
  const [expandedProjects, setExpandedProjects] = useState<Set<string>>(new Set());
  const [projectsByWs, setProjectsByWs] = useState<Record<string, Project[]>>({});
  const [modulesByProj, setModulesByProj] = useState<Record<string, Module[]>>({});
  const [hoveredItemId, setHoveredItemId] = useState<string | null>(null);

  // CRUD state
  const [creating, setCreating] = useState<{ level: "workspace" | "project" | "module"; parentId?: string } | null>(null);
  const [newName, setNewName] = useState("");
  const [templateModuleId, setTemplateModuleId] = useState("");
  const [allModules, setAllModules] = useState<Module[]>([]);
  const [renamingId, setRenamingId] = useState<string | null>(null);
  const [renameName, setRenameName] = useState("");

  // Fetch workspaces on mount
  useEffect(() => {
    api.listWorkspaces().then((data) => setWorkspaces(data.items)).catch(() => {});
  }, []);

  // Auto-expand path from URL params
  useEffect(() => {
    if (params.workspaceId) {
      setExpandedWorkspaces((prev) => new Set(prev).add(params.workspaceId!));
      if (!projectsByWs[params.workspaceId]) {
        fetchProjects(params.workspaceId);
      }
    }
    if (params.projectId) {
      setExpandedProjects((prev) => new Set(prev).add(params.projectId!));
      if (!modulesByProj[params.projectId]) {
        fetchModules(params.projectId);
      }
    }
  }, [params.workspaceId, params.projectId]);

  const fetchProjects = async (wsId: string) => {
    try {
      const data = await api.listProjects(wsId);
      setProjectsByWs((prev) => ({ ...prev, [wsId]: data.items }));
    } catch { /* ignore */ }
  };

  const fetchModules = async (projId: string) => {
    try {
      const data = await api.listModules({ project_id: projId });
      setModulesByProj((prev) => ({ ...prev, [projId]: data.items }));
    } catch { /* ignore */ }
  };

  const toggleWorkspace = (wsId: string) => {
    setExpandedWorkspaces((prev) => {
      const next = new Set(prev);
      if (next.has(wsId)) {
        next.delete(wsId);
      } else {
        next.add(wsId);
        if (!projectsByWs[wsId]) fetchProjects(wsId);
      }
      return next;
    });
  };

  const toggleProject = (projId: string) => {
    setExpandedProjects((prev) => {
      const next = new Set(prev);
      if (next.has(projId)) {
        next.delete(projId);
      } else {
        next.add(projId);
        if (!modulesByProj[projId]) fetchModules(projId);
      }
      return next;
    });
  };

  const isSelected = (kind: string, id: string) => {
    if (kind === "workspace") return params.workspaceId === id && !params.projectId;
    if (kind === "project") return params.projectId === id && !params.moduleId;
    if (kind === "module") return params.moduleId === id;
    return false;
  };

  const isTraceabilitySelected = (projId: string) =>
    params.projectId === projId && window.location.pathname.endsWith("/traceability");

  // CRUD handlers
  const handleCreate = async () => {
    if (!newName.trim() || !creating) return;
    try {
      if (creating.level === "workspace") {
        await api.createWorkspace({ name: newName.trim() });
        const data = await api.listWorkspaces();
        setWorkspaces(data.items);
      } else if (creating.level === "project" && creating.parentId) {
        await api.createProject(creating.parentId, { name: newName.trim() });
        await fetchProjects(creating.parentId);
      } else if (creating.level === "module" && creating.parentId) {
        if (templateModuleId) {
          await api.createModuleFromTemplate({
            name: newName.trim(),
            project_id: creating.parentId,
            template_module_id: templateModuleId,
          });
        } else {
          await api.createModule({ name: newName.trim(), project_id: creating.parentId });
        }
        await fetchModules(creating.parentId);
      }
    } catch { /* ignore */ }
    setCreating(null);
    setNewName("");
    setTemplateModuleId("");
  };

  const handleRename = async (id: string, level: "workspace" | "project" | "module", parentId?: string) => {
    if (!renameName.trim()) return;
    try {
      if (level === "workspace") {
        await api.updateWorkspace(id, { name: renameName.trim() });
        const data = await api.listWorkspaces();
        setWorkspaces(data.items);
      } else if (level === "project" && parentId) {
        await api.updateProject(parentId, id, { name: renameName.trim() });
        await fetchProjects(parentId);
      } else if (level === "module") {
        await api.updateModule(id, { name: renameName.trim() });
        for (const [projId, mods] of Object.entries(modulesByProj)) {
          if (mods.some((m) => m.id === id)) {
            await fetchModules(projId);
            break;
          }
        }
      }
    } catch { /* ignore */ }
    setRenamingId(null);
    setRenameName("");
  };

  const handleDelete = async (id: string, level: "workspace" | "project" | "module", parentId?: string) => {
    try {
      if (level === "workspace") {
        await api.deleteWorkspace(id);
        const data = await api.listWorkspaces();
        setWorkspaces(data.items);
        if (params.workspaceId === id) navigate("/");
      } else if (level === "project" && parentId) {
        await api.deleteProject(parentId, id);
        await fetchProjects(parentId);
        if (params.projectId === id) navigate(`/w/${parentId}`);
      } else if (level === "module") {
        await api.deleteModule(id);
        for (const [projId, mods] of Object.entries(modulesByProj)) {
          if (mods.some((m) => m.id === id)) {
            await fetchModules(projId);
            break;
          }
        }
        if (params.moduleId === id) navigate("/");
      }
    } catch { /* ignore */ }
  };

  const itemStyle = (level: number, selected: boolean): React.CSSProperties => ({
    display: "flex",
    alignItems: "center",
    padding: theme.sidebar.itemPadding,
    paddingLeft: level * theme.sidebar.indent + 8,
    cursor: "pointer",
    background: selected ? "#e3f2fd" : "transparent",
    borderRadius: theme.borderRadius,
    fontSize: "0.875rem",
    userSelect: "none",
  });

  const chevronStyle: React.CSSProperties = {
    width: 16,
    textAlign: "center",
    marginRight: 4,
    fontSize: "0.7rem",
    color: theme.colors.textSecondary,
    flexShrink: 0,
  };

  const actionBtnStyle: React.CSSProperties = {
    background: "none",
    border: "none",
    cursor: "pointer",
    padding: "0 2px",
    fontSize: "0.75rem",
    color: theme.colors.textSecondary,
    lineHeight: 1,
  };

  const inlineInputStyle: React.CSSProperties = {
    fontSize: "0.85rem",
    padding: "2px 4px",
    border: `1px solid ${theme.colors.border}`,
    borderRadius: theme.borderRadius,
    outline: "none",
    width: "100%",
  };

  if (collapsed) {
    return (
      <div
        style={{
          width: 36,
          borderRight: `1px solid ${theme.colors.sidebarBorder}`,
          background: theme.colors.sidebarBg,
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          paddingTop: theme.spacing.sm,
          flexShrink: 0,
        }}
      >
        <button
          onClick={onToggleCollapse}
          style={{ background: "none", border: "none", cursor: "pointer", fontSize: "1rem" }}
          title="Expand sidebar"
        >
          &#9654;
        </button>
      </div>
    );
  }

  return (
    <div
      style={{
        width: theme.sidebar.width,
        borderRight: `1px solid ${theme.colors.sidebarBorder}`,
        background: theme.colors.sidebarBg,
        overflowY: "auto",
        overflowX: "hidden",
        flexShrink: 0,
        display: "flex",
        flexDirection: "column",
      }}
    >
      {/* Sidebar header */}
      <div
        style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          padding: theme.sidebar.itemPadding,
          borderBottom: `1px solid ${theme.colors.sidebarBorder}`,
        }}
      >
        <span style={{ fontWeight: 600, fontSize: "0.85rem" }}>Navigation</span>
        <div style={{ display: "flex", gap: 4 }}>
          <button
            onClick={() => { setCreating({ level: "workspace" }); setNewName(""); }}
            style={actionBtnStyle}
            title="New workspace"
          >
            +
          </button>
          <button
            onClick={onToggleCollapse}
            style={actionBtnStyle}
            title="Collapse sidebar"
          >
            &#9664;
          </button>
        </div>
      </div>

      {/* Tree */}
      <div style={{ flex: 1, padding: `${theme.spacing.xs} 0` }}>
        {/* Inline create workspace */}
        {creating?.level === "workspace" && (
          <div style={{ padding: theme.sidebar.itemPadding, paddingLeft: 8 }}>
            <input
              value={newName}
              onChange={(e) => setNewName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleCreate();
                if (e.key === "Escape") { setCreating(null); setNewName(""); }
              }}
              placeholder="Workspace name"
              style={inlineInputStyle}
              autoFocus
            />
          </div>
        )}

        {workspaces.map((ws) => {
          const wsExpanded = expandedWorkspaces.has(ws.id);
          const projects = projectsByWs[ws.id] ?? [];

          return (
            <div key={ws.id}>
              {/* Workspace row */}
              <div
                style={itemStyle(0, isSelected("workspace", ws.id))}
                onMouseEnter={() => setHoveredItemId(`ws-${ws.id}`)}
                onMouseLeave={() => setHoveredItemId(null)}
              >
                <span style={chevronStyle} onClick={() => toggleWorkspace(ws.id)}>
                  {wsExpanded ? "\u25BC" : "\u25B6"}
                </span>
                {renamingId === `ws-${ws.id}` ? (
                  <input
                    value={renameName}
                    onChange={(e) => setRenameName(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") handleRename(ws.id, "workspace");
                      if (e.key === "Escape") setRenamingId(null);
                    }}
                    style={{ ...inlineInputStyle, flex: 1 }}
                    autoFocus
                    onClick={(e) => e.stopPropagation()}
                  />
                ) : (
                  <span
                    style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}
                    onClick={() => navigate(`/w/${ws.id}`)}
                  >
                    {ws.name}
                  </span>
                )}
                {hoveredItemId === `ws-${ws.id}` && renamingId !== `ws-${ws.id}` && (
                  <span style={{ display: "flex", gap: 2, marginLeft: 4 }}>
                    <button
                      style={actionBtnStyle}
                      title="Add project"
                      onClick={(e) => { e.stopPropagation(); setCreating({ level: "project", parentId: ws.id }); setNewName(""); if (!wsExpanded) toggleWorkspace(ws.id); }}
                    >
                      +
                    </button>
                    <button
                      style={actionBtnStyle}
                      title="Rename"
                      onClick={(e) => { e.stopPropagation(); setRenamingId(`ws-${ws.id}`); setRenameName(ws.name); }}
                    >
                      &#9998;
                    </button>
                    <button
                      style={{ ...actionBtnStyle, color: theme.colors.danger }}
                      title="Delete"
                      onClick={(e) => { e.stopPropagation(); handleDelete(ws.id, "workspace"); }}
                    >
                      &#10005;
                    </button>
                  </span>
                )}
              </div>

              {/* Expanded workspace children */}
              {wsExpanded && (
                <>
                  {/* Inline create project */}
                  {creating?.level === "project" && creating.parentId === ws.id && (
                    <div style={{ padding: theme.sidebar.itemPadding, paddingLeft: 1 * theme.sidebar.indent + 8 }}>
                      <input
                        value={newName}
                        onChange={(e) => setNewName(e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === "Enter") handleCreate();
                          if (e.key === "Escape") { setCreating(null); setNewName(""); }
                        }}
                        placeholder="Project name"
                        style={inlineInputStyle}
                        autoFocus
                      />
                    </div>
                  )}

                  {projects.map((proj) => {
                    const projExpanded = expandedProjects.has(proj.id);
                    const mods = modulesByProj[proj.id] ?? [];

                    return (
                      <div key={proj.id}>
                        {/* Project row */}
                        <div
                          style={itemStyle(1, isSelected("project", proj.id))}
                          onMouseEnter={() => setHoveredItemId(`proj-${proj.id}`)}
                          onMouseLeave={() => setHoveredItemId(null)}
                        >
                          <span style={chevronStyle} onClick={() => toggleProject(proj.id)}>
                            {projExpanded ? "\u25BC" : "\u25B6"}
                          </span>
                          {renamingId === `proj-${proj.id}` ? (
                            <input
                              value={renameName}
                              onChange={(e) => setRenameName(e.target.value)}
                              onKeyDown={(e) => {
                                if (e.key === "Enter") handleRename(proj.id, "project", ws.id);
                                if (e.key === "Escape") setRenamingId(null);
                              }}
                              style={{ ...inlineInputStyle, flex: 1 }}
                              autoFocus
                              onClick={(e) => e.stopPropagation()}
                            />
                          ) : (
                            <span
                              style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}
                              onClick={() => navigate(`/w/${ws.id}/p/${proj.id}`)}
                            >
                              {proj.name}
                            </span>
                          )}
                          {hoveredItemId === `proj-${proj.id}` && renamingId !== `proj-${proj.id}` && (
                            <span style={{ display: "flex", gap: 2, marginLeft: 4 }}>
                              <button
                                style={actionBtnStyle}
                                title="Add module"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  setCreating({ level: "module", parentId: proj.id });
                                  setNewName("");
                                  setTemplateModuleId("");
                                  api.listModules({ limit: 500 }).then((d) => setAllModules(d.items)).catch(() => {});
                                  if (!projExpanded) toggleProject(proj.id);
                                }}
                              >
                                +
                              </button>
                              <button
                                style={actionBtnStyle}
                                title="Rename"
                                onClick={(e) => { e.stopPropagation(); setRenamingId(`proj-${proj.id}`); setRenameName(proj.name); }}
                              >
                                &#9998;
                              </button>
                              <button
                                style={{ ...actionBtnStyle, color: theme.colors.danger }}
                                title="Delete"
                                onClick={(e) => { e.stopPropagation(); handleDelete(proj.id, "project", ws.id); }}
                              >
                                &#10005;
                              </button>
                            </span>
                          )}
                        </div>

                        {/* Expanded project children */}
                        {projExpanded && (
                          <>
                            {/* Traceability Matrix item */}
                            <div
                              style={{
                                ...itemStyle(2, isTraceabilitySelected(proj.id)),
                                fontStyle: "italic",
                                fontSize: "0.8rem",
                                color: theme.colors.textSecondary,
                              }}
                              onClick={() => navigate(`/w/${ws.id}/p/${proj.id}/traceability`)}
                            >
                              <span style={{ ...chevronStyle, visibility: "hidden" }} />
                              Traceability Matrix
                            </div>

                            {/* Inline create module */}
                            {creating?.level === "module" && creating.parentId === proj.id && (
                              <div style={{ padding: theme.sidebar.itemPadding, paddingLeft: 2 * theme.sidebar.indent + 8, display: "flex", flexDirection: "column", gap: 4 }}>
                                <input
                                  value={newName}
                                  onChange={(e) => setNewName(e.target.value)}
                                  onKeyDown={(e) => {
                                    if (e.key === "Enter") handleCreate();
                                    if (e.key === "Escape") { setCreating(null); setNewName(""); setTemplateModuleId(""); }
                                  }}
                                  placeholder="Module name"
                                  style={inlineInputStyle}
                                  autoFocus
                                />
                                {allModules.length > 0 && (
                                  <select
                                    value={templateModuleId}
                                    onChange={(e) => setTemplateModuleId(e.target.value)}
                                    style={{ ...inlineInputStyle, cursor: "pointer" }}
                                  >
                                    <option value="">(blank module)</option>
                                    {allModules.map((m) => (
                                      <option key={m.id} value={m.id}>
                                        Template: {m.name}
                                      </option>
                                    ))}
                                  </select>
                                )}
                              </div>
                            )}

                            {mods.map((mod) => (
                              <div
                                key={mod.id}
                                style={itemStyle(2, isSelected("module", mod.id))}
                                onMouseEnter={() => setHoveredItemId(`mod-${mod.id}`)}
                                onMouseLeave={() => setHoveredItemId(null)}
                              >
                                <span style={{ ...chevronStyle, visibility: "hidden" }} />
                                {renamingId === `mod-${mod.id}` ? (
                                  <input
                                    value={renameName}
                                    onChange={(e) => setRenameName(e.target.value)}
                                    onKeyDown={(e) => {
                                      if (e.key === "Enter") handleRename(mod.id, "module");
                                      if (e.key === "Escape") setRenamingId(null);
                                    }}
                                    style={{ ...inlineInputStyle, flex: 1 }}
                                    autoFocus
                                    onClick={(e) => e.stopPropagation()}
                                  />
                                ) : (
                                  <span
                                    style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}
                                    onClick={() => navigate(`/w/${ws.id}/p/${proj.id}/m/${mod.id}`)}
                                  >
                                    {mod.name}
                                  </span>
                                )}
                                {hoveredItemId === `mod-${mod.id}` && renamingId !== `mod-${mod.id}` && (
                                  <span style={{ display: "flex", gap: 2, marginLeft: 4 }}>
                                    <button
                                      style={actionBtnStyle}
                                      title="Rename"
                                      onClick={(e) => { e.stopPropagation(); setRenamingId(`mod-${mod.id}`); setRenameName(mod.name); }}
                                    >
                                      &#9998;
                                    </button>
                                    <button
                                      style={{ ...actionBtnStyle, color: theme.colors.danger }}
                                      title="Delete"
                                      onClick={(e) => { e.stopPropagation(); handleDelete(mod.id, "module"); }}
                                    >
                                      &#10005;
                                    </button>
                                  </span>
                                )}
                              </div>
                            ))}
                          </>
                        )}
                      </div>
                    );
                  })}
                </>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AgGridReact } from "ag-grid-react";
import {
  AllCommunityModule,
  type CellValueChangedEvent,
  type ColDef,
  type ICellRendererParams,
  themeQuartz,
} from "ag-grid-community";
import {
  api,
  isReviewed,
  type AttributeDefinition,
  type Module,
  type ObjectType,
  type ReqObject,
} from "./api/client";
import Markdown from "react-markdown";
import { ObjectHistory } from "./ObjectHistory";
import { LinkPanel } from "./LinkPanel";
import { BaselinePanel } from "./BaselinePanel";
import { AttributeDefPanel } from "./AttributeDefPanel";
import { ScriptPanel } from "./ScriptPanel";
import { ValidationPanel } from "./ValidationPanel";
import { ModuleSettings } from "./ModuleSettings";
import { ObjectTree } from "./ObjectTree";
import { ViewBar } from "./ViewBar";
import { CommentPanel } from "./CommentPanel";
import { ObjectTypePanel } from "./ObjectTypePanel";
import { ImpactPanel } from "./ImpactPanel";
import { CoverageWidget } from "./CoverageWidget";
import { theme } from "./theme";

interface Props {
  module: Module;
  onModuleUpdated: (m: Module) => void;
}

type Tab = "objects" | "links" | "baselines" | "attributes" | "scripts" | "validation" | "types" | "settings";

const TABS: { key: Tab; label: string }[] = [
  { key: "objects", label: "Objects" },
  { key: "links", label: "Links" },
  { key: "baselines", label: "Baselines" },
  { key: "attributes", label: "Attributes" },
  { key: "scripts", label: "Scripts" },
  { key: "validation", label: "Validation" },
  { key: "types", label: "Types" },
  { key: "settings", label: "Settings" },
];

type PanelState =
  | { type: "history"; objectId: string }
  | { type: "comments"; objectId: string }
  | { type: "impact"; objectId: string }
  | null;

export function ModuleDetail({ module, onModuleUpdated }: Props) {
  const [objects, setObjects] = useState<ReqObject[]>([]);
  const [attrDefs, setAttrDefs] = useState<AttributeDefinition[]>([]);
  const [objectTypes, setObjectTypes] = useState<ObjectType[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [heading, setHeading] = useState("");
  const [body, setBody] = useState("");
  const [createTypeId, setCreateTypeId] = useState("");
  const [activePanel, setActivePanel] = useState<PanelState>(null);
  const [searchQuery, setSearchQuery] = useState("");
  const [activeSearch, setActiveSearch] = useState("");
  const [filterHeading, setFilterHeading] = useState("");
  const [filterBody, setFilterBody] = useState("");
  const [needsReview, setNeedsReview] = useState(false);
  const [activeTab, setActiveTab] = useState<Tab>("objects");
  const [collapsedIds, setCollapsedIds] = useState<Set<string>>(new Set());
  const [gridReady, setGridReady] = useState(false);
  const [layoutColumns, setLayoutColumns] = useState<Map<string, Map<string, string>>>(new Map());
  const gridRef = useRef<AgGridReact<ReqObject>>(null);

  // Reset tab when module changes
  useEffect(() => {
    setActiveTab("objects");
    setActivePanel(null);
    setGridReady(false);
  }, [module.id]);

  const fetchObjects = useCallback(async () => {
    try {
      const filters: Record<string, string> = {};
      if (filterHeading) filters.heading = filterHeading;
      if (filterBody) filters.body = filterBody;
      if (activeSearch) filters.search = activeSearch;
      if (needsReview) filters.needs_review = "true";
      const data = await api.listObjects(
        module.id,
        Object.keys(filters).length > 0 ? filters : undefined,
      );
      setObjects(data.items);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load objects");
    }
  }, [module.id, filterHeading, filterBody, activeSearch, needsReview]);

  const fetchAttrDefs = useCallback(async () => {
    try {
      const data = await api.listAttributeDefinitions(module.id);
      setAttrDefs(data.items);
    } catch {
      // Non-critical — grid still works without attribute columns
    }
  }, [module.id]);

  const fetchObjectTypes = useCallback(async () => {
    try {
      const data = await api.listObjectTypes(module.id);
      setObjectTypes(data.items);
    } catch {
      // Non-critical
    }
  }, [module.id]);

  useEffect(() => {
    fetchObjects();
    fetchAttrDefs();
    fetchObjectTypes();
  }, [fetchObjects, fetchAttrDefs, fetchObjectTypes]);

  // Fetch layout columns from enabled layout scripts
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const scripts = await api.listScripts(module.id);
        const layoutScripts = scripts.filter(
          (s) => s.script_type === "layout" && s.enabled,
        );
        const newMap = new Map<string, Map<string, string>>();
        for (const script of layoutScripts) {
          try {
            const res = await api.batchLayout(module.id, script.id);
            const objMap = new Map<string, string>();
            for (const entry of res.results) {
              objMap.set(entry.object_id, entry.value);
            }
            newMap.set(script.name, objMap);
          } catch {
            // Skip failing scripts
          }
        }
        if (!cancelled) setLayoutColumns(newMap);
      } catch {
        // Non-critical
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [module.id]);

  // Build a set of IDs that have children for expand/collapse UI
  const childrenMap = useMemo(() => {
    const parents = new Set<string>();
    for (const obj of objects) {
      if (obj.parent_id) parents.add(obj.parent_id);
    }
    return parents;
  }, [objects]);

  // Filter out children of collapsed parents
  const visibleObjects = useMemo(() => {
    if (collapsedIds.size === 0) return objects;
    return objects.filter((obj) => {
      // Walk up the level hierarchy to see if any ancestor is collapsed
      const parts = obj.level.split(".");
      for (let i = 1; i < parts.length; i++) {
        const ancestorLevel = parts.slice(0, i).join(".");
        const ancestor = objects.find((o) => o.level === ancestorLevel);
        if (ancestor && collapsedIds.has(ancestor.id)) return false;
      }
      return true;
    });
  }, [objects, collapsedIds]);

  const toggleCollapse = useCallback((id: string) => {
    setCollapsedIds((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  const handleSearch = () => {
    setActiveSearch(searchQuery.trim());
  };

  const handleTreeNodeSelect = useCallback(
    (id: string) => {
      setActivePanel({ type: "history", objectId: id });
      const gridApi = gridRef.current?.api;
      if (gridApi) {
        const rowNode = gridApi.getRowNode(id);
        if (rowNode) {
          gridApi.ensureNodeVisible(rowNode);
          gridApi.setNodesSelected({ nodes: [rowNode], newValue: true });
        }
      }
    },
    [],
  );

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!heading.trim() && !body.trim()) return;
    try {
      await api.createObject(module.id, {
        heading: heading.trim() || undefined,
        body: body.trim() || undefined,
        object_type_id: createTypeId || undefined,
      });
      setHeading("");
      setBody("");
      setCreateTypeId("");
      fetchObjects();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create object");
    }
  };

  const handleDelete = useCallback(
    async (id: string) => {
      try {
        await api.deleteObject(module.id, id);
        if (activePanel?.objectId === id) setActivePanel(null);
        fetchObjects();
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "Failed to delete object",
        );
      }
    },
    [module.id, fetchObjects, activePanel],
  );

  const handleCellValueChanged = useCallback(
    async (event: CellValueChangedEvent<ReqObject>) => {
      const { data, colDef } = event;
      if (!data || !colDef.field) return;

      try {
        // Check if this is an attribute column (prefixed with "attr.")
        if (colDef.field.startsWith("attr.")) {
          const attrName = colDef.field.slice(5);
          const currentAttrs =
            (data.attributes as Record<string, unknown>) ?? {};
          const newVal = event.newValue;
          const updatedAttrs = { ...currentAttrs, [attrName]: newVal };
          await api.updateObject(module.id, data.id, {
            attributes: updatedAttrs as Record<string, unknown>,
          });
        } else {
          await api.updateObject(module.id, data.id, {
            [colDef.field]: data[colDef.field as keyof ReqObject],
          });
        }
        fetchObjects();
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "Failed to update object",
        );
        fetchObjects(); // Revert grid to server state
      }
    },
    [module.id, fetchObjects],
  );

  const handleToggleReview = useCallback(
    async (obj: ReqObject) => {
      try {
        await api.updateObject(module.id, obj.id, {
          reviewed: !isReviewed(obj),
        });
        fetchObjects();
      } catch (err) {
        setError(
          err instanceof Error ? err.message : "Failed to update review",
        );
      }
    },
    [module.id, fetchObjects],
  );

  // Build dynamic attribute columns from definitions
  const attrColumns = useMemo<ColDef<ReqObject>[]>(() => {
    return attrDefs.map((def) => {
      const col: ColDef<ReqObject> = {
        headerName: def.name,
        field: `attr.${def.name}` as keyof ReqObject & string,
        editable: true,
        width: 130,
        valueGetter: (p) => {
          const attrs = p.data?.attributes as Record<string, unknown> | null;
          return attrs?.[def.name] ?? null;
        },
        valueSetter: (p) => {
          if (!p.data) return false;
          const attrs =
            (p.data.attributes as Record<string, unknown>) ?? {};
          (p.data as unknown as Record<string, unknown>).attributes = {
            ...attrs,
            [def.name]: p.newValue,
          };
          return true;
        },
      };

      // Use dropdown editor for enum and bool types
      if (def.data_type === "enum" && Array.isArray(def.enum_values)) {
        col.cellEditor = "agSelectCellEditor";
        col.cellEditorParams = { values: def.enum_values };
      } else if (def.data_type === "bool") {
        col.cellEditor = "agSelectCellEditor";
        col.cellEditorParams = { values: ["true", "false"] };
      }

      return col;
    });
  }, [attrDefs]);

  // Build computed layout columns from layout scripts
  const layoutColDefs = useMemo<ColDef<ReqObject>[]>(() => {
    const cols: ColDef<ReqObject>[] = [];
    for (const [scriptName, objMap] of layoutColumns) {
      cols.push({
        headerName: scriptName,
        width: 130,
        editable: false,
        sortable: false,
        cellStyle: { fontStyle: "italic" },
        valueGetter: (p) => {
          if (!p.data) return null;
          return objMap.get(p.data.id) ?? "";
        },
      });
    }
    return cols;
  }, [layoutColumns]);

  const columnDefs = useMemo<ColDef<ReqObject>[]>(
    () => [
      { headerName: "Level", field: "level", width: 90 },
      {
        headerName: "Heading",
        field: "heading",
        flex: 1,
        editable: true,
        cellRenderer: (p: ICellRendererParams<ReqObject>) => {
          if (!p.data) return null;
          const depth = (p.data.level?.split(".").length ?? 1) - 1;
          const hasKids = childrenMap.has(p.data.id);
          const isCollapsed = collapsedIds.has(p.data.id);
          return (
            <span style={{ paddingLeft: `${depth * 20}px`, display: "inline-flex", alignItems: "center", gap: "4px" }}>
              {hasKids ? (
                <span
                  style={{
                    cursor: "pointer",
                    userSelect: "none",
                    width: "16px",
                    textAlign: "center",
                    fontSize: "0.75em",
                    display: "inline-block",
                    transition: "transform 0.15s",
                    transform: isCollapsed ? "rotate(-90deg)" : "rotate(0deg)",
                  }}
                  onClick={(e) => { e.stopPropagation(); toggleCollapse(p.data!.id); }}
                >
                  {"\u25BC"}
                </span>
              ) : (
                <span style={{ width: "16px" }} />
              )}
              <span>{p.value ?? ""}</span>
            </span>
          );
        },
      },
      {
        headerName: "Body",
        field: "body",
        flex: 2,
        editable: true,
        autoHeight: true,
        wrapText: true,
        cellRenderer: (p: ICellRendererParams<ReqObject>) =>
          p.data?.body ? <Markdown>{p.data.body}</Markdown> : null,
      },
      {
        headerName: "Classification",
        field: "classification",
        width: 130,
        editable: true,
        cellEditor: "agSelectCellEditor",
        cellEditorParams: { values: ["normative", "informative", "heading"] },
        cellStyle: (params) => {
          const c = params.value;
          if (c === "informative") return { color: "#1565c0" };
          if (c === "heading") return { color: "#6a1b9a" };
          return null;
        },
      },
      ...attrColumns,
      ...layoutColDefs,
      {
        headerName: "Reviewed",
        width: 100,
        sortable: false,
        cellRenderer: (p: ICellRendererParams<ReqObject>) =>
          p.data ? (
            <span
              style={{ cursor: "pointer", fontSize: "1.2em" }}
              onClick={(e) => {
                e.stopPropagation();
                handleToggleReview(p.data!);
              }}
            >
              {isReviewed(p.data) ? "\u2705" : "\u274C"}
            </span>
          ) : null,
      },
      { headerName: "Version", field: "current_version", width: 100 },
      {
        headerName: "Updated",
        field: "updated_at",
        width: 160,
        valueFormatter: (p) =>
          p.value ? new Date(p.value as string).toLocaleString() : "",
      },
      {
        headerName: "",
        width: 160,
        sortable: false,
        filter: false,
        cellRenderer: (p: ICellRendererParams<ReqObject>) =>
          p.data ? (
            <span style={{ display: "inline-flex", gap: "4px" }}>
              <button
                title="Comments"
                onClick={(e) => {
                  e.stopPropagation();
                  setActivePanel({ type: "comments", objectId: p.data!.id });
                }}
                style={{ padding: "2px 6px", fontSize: "0.8rem" }}
              >
                Cmt
              </button>
              <button
                title="Impact Analysis"
                onClick={(e) => {
                  e.stopPropagation();
                  setActivePanel({ type: "impact", objectId: p.data!.id });
                }}
                style={{ padding: "2px 6px", fontSize: "0.8rem" }}
              >
                Imp
              </button>
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  handleDelete(p.data!.id);
                }}
                style={{ padding: "2px 6px", fontSize: "0.8rem" }}
              >
                Del
              </button>
            </span>
          ) : null,
      },
    ],
    [handleDelete, handleToggleReview, attrColumns, layoutColDefs, childrenMap, collapsedIds, toggleCollapse],
  );

  const tabStyle = (tab: Tab): React.CSSProperties => ({
    padding: `${theme.spacing.sm} ${theme.spacing.md}`,
    border: "none",
    background: "none",
    cursor: "pointer",
    fontSize: "0.9rem",
    color: activeTab === tab ? theme.colors.tabActive : theme.colors.tabInactive,
    borderBottom: activeTab === tab ? `2px solid ${theme.colors.tabActive}` : "2px solid transparent",
    fontWeight: activeTab === tab ? 600 : 400,
  });

  const handlePublishHtml = () => {
    window.open(api.getPublishUrl(module.id, "html"), "_blank");
  };

  return (
    <div>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <h1 style={{ marginTop: 0 }}>{module.name}</h1>
        <button
          onClick={handlePublishHtml}
          style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}`, height: "fit-content" }}
        >
          Publish HTML
        </button>
      </div>

      {error && (
        <div style={{ color: theme.colors.error, marginBottom: theme.spacing.md }}>{error}</div>
      )}

      {/* Tab bar */}
      <div style={{ display: "flex", borderBottom: `1px solid ${theme.colors.borderLight}`, marginBottom: theme.spacing.md }}>
        {TABS.map((t) => (
          <button key={t.key} style={tabStyle(t.key)} onClick={() => setActiveTab(t.key)}>
            {t.label}
          </button>
        ))}
      </div>

      {/* Tab content */}
      {activeTab === "objects" && (
        <>
          {/* Search bar */}
          <div style={{ display: "flex", gap: theme.spacing.sm, marginBottom: "0.75rem" }}>
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleSearch();
              }}
              placeholder="Full-text search..."
              style={{ padding: theme.spacing.sm, flex: 1 }}
            />
            <button onClick={handleSearch} style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}>
              Search
            </button>
            {activeSearch && (
              <button
                onClick={() => {
                  setSearchQuery("");
                  setActiveSearch("");
                }}
                style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}
              >
                Clear
              </button>
            )}
          </div>

          {/* Filters */}
          <div style={{ display: "flex", gap: theme.spacing.sm, marginBottom: "0.75rem", alignItems: "center" }}>
            <input
              type="text"
              value={filterHeading}
              onChange={(e) => setFilterHeading(e.target.value)}
              placeholder="Filter heading..."
              style={{ padding: theme.spacing.sm, flex: 1 }}
            />
            <input
              type="text"
              value={filterBody}
              onChange={(e) => setFilterBody(e.target.value)}
              placeholder="Filter body..."
              style={{ padding: theme.spacing.sm, flex: 1 }}
            />
            <label style={{ display: "flex", alignItems: "center", gap: "4px", whiteSpace: "nowrap" }}>
              <input
                type="checkbox"
                checked={needsReview}
                onChange={(e) => setNeedsReview(e.target.checked)}
              />
              Needs Review
            </label>
          </div>

          <CoverageWidget moduleId={module.id} />

          <form
            onSubmit={handleCreate}
            style={{ marginBottom: theme.spacing.md, display: "flex", gap: theme.spacing.sm }}
          >
            <input
              type="text"
              value={heading}
              onChange={(e) => setHeading(e.target.value)}
              placeholder="Heading (e.g. REQ-001)"
              style={{ padding: theme.spacing.sm, flex: 1 }}
            />
            <input
              type="text"
              value={body}
              onChange={(e) => setBody(e.target.value)}
              placeholder="Body"
              style={{ padding: theme.spacing.sm, flex: 2 }}
            />
            {objectTypes.length > 0 && (
              <select
                value={createTypeId}
                onChange={(e) => setCreateTypeId(e.target.value)}
                style={{ padding: theme.spacing.sm }}
              >
                <option value="">(no type)</option>
                {objectTypes.map((t) => (
                  <option key={t.id} value={t.id}>
                    {t.name}
                  </option>
                ))}
              </select>
            )}
            <button type="submit" style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}` }}>
              Add
            </button>
          </form>

          <ViewBar moduleId={module.id} gridRef={gridRef} gridReady={gridReady} />

          <div style={{ display: "flex", height: 500 }}>
            <ObjectTree
              objects={objects}
              selectedId={activePanel?.objectId ?? null}
              onSelect={handleTreeNodeSelect}
            />
            <div style={{ flex: 1 }}>
              <AgGridReact<ReqObject>
                ref={gridRef}
                theme={themeQuartz}
                modules={[AllCommunityModule]}
                rowData={visibleObjects}
                columnDefs={columnDefs}
                getRowId={(p) => p.data.id}
                onCellValueChanged={handleCellValueChanged}
                onGridReady={() => setGridReady(true)}
                onRowClicked={(e) => {
                  const target = e.event?.target as HTMLElement | undefined;
                  if (target?.closest("button")) return;
                  if (e.data) setActivePanel({ type: "history", objectId: e.data.id });
                }}
              />
            </div>
          </div>
        </>
      )}

      {activeTab === "links" && (
        <LinkPanel moduleId={module.id} objects={objects} />
      )}

      {activeTab === "baselines" && (
        <BaselinePanel moduleId={module.id} />
      )}

      {activeTab === "attributes" && (
        <AttributeDefPanel moduleId={module.id} onDefsChanged={fetchAttrDefs} />
      )}

      {activeTab === "scripts" && (
        <ScriptPanel moduleId={module.id} />
      )}

      {activeTab === "validation" && (
        <ValidationPanel moduleId={module.id} />
      )}

      {activeTab === "types" && (
        <ObjectTypePanel moduleId={module.id} />
      )}

      {activeTab === "settings" && (
        <ModuleSettings module={module} onModuleUpdated={onModuleUpdated} />
      )}

      {/* Panels — shown regardless of active tab */}
      {activePanel?.type === "history" && (
        <ObjectHistory
          moduleId={module.id}
          objectId={activePanel.objectId}
          onClose={() => setActivePanel(null)}
        />
      )}
      {activePanel?.type === "comments" && (
        <CommentPanel
          objectId={activePanel.objectId}
          onClose={() => setActivePanel(null)}
        />
      )}
      {activePanel?.type === "impact" && (
        <ImpactPanel
          objectId={activePanel.objectId}
          onClose={() => setActivePanel(null)}
        />
      )}
    </div>
  );
}

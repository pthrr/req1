import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { AgGridReact } from "ag-grid-react";
import {
  AllCommunityModule,
  type CellValueChangedEvent,
  type ColDef,
  type ICellRendererParams,
  type RowDragEndEvent,
  themeQuartz,
} from "ag-grid-community";
import {
  api,
  isReviewed,
  type AttributeDefinition,
  type Link,
  type Module,
  type ObjectType,
  type ReqObject,
} from "./api/client";
import Markdown from "react-markdown";
import { useNavigate } from "react-router";
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
import { ObjectDetailPanel } from "./ObjectDetailPanel";
import { PublishPreviewPanel } from "./PublishPreviewPanel";
import { ReviewPanel } from "./ReviewPanel";
import { ChangeProposalPanel } from "./ChangeProposalPanel";
import { ReferencesPanel } from "./ReferencesPanel";
import { ReviewDiffPanel } from "./ReviewDiffPanel";
import { ActivityFeed } from "./ActivityFeed";
import { theme } from "./theme";

interface Props {
  module: Module;
  onModuleUpdated: (m: Module) => void;
}

type Tab = "objects" | "links" | "baselines" | "attributes" | "scripts" | "validation" | "types" | "settings" | "reviews" | "proposals";

const TABS: { key: Tab; label: string }[] = [
  { key: "objects", label: "Objects" },
  { key: "links", label: "Links" },
  { key: "baselines", label: "Baselines" },
  { key: "attributes", label: "Attributes" },
  { key: "scripts", label: "Scripts" },
  { key: "validation", label: "Validation" },
  { key: "types", label: "Types" },
  { key: "reviews", label: "Reviews" },
  { key: "proposals", label: "Proposals" },
  { key: "settings", label: "Settings" },
];

const PUBLISH_FORMATS = [
  { value: "html", label: "HTML", enabled: true },
  { value: "markdown", label: "Markdown", enabled: true },
  { value: "latex", label: "LaTeX", enabled: false },
  { value: "plaintext", label: "Plain Text", enabled: false },
];

type PanelState =
  | { type: "history"; objectId: string }
  | { type: "comments"; objectId: string }
  | { type: "impact"; objectId: string }
  | { type: "detail"; objectId: string }
  | { type: "references"; objectId: string }
  | { type: "reviewDiff"; objectId: string }
  | { type: "preview" }
  | null;

export function ModuleDetail({ module, onModuleUpdated }: Props) {
  const navigate = useNavigate();
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
  const [selectedObjectId, setSelectedObjectId] = useState<string | null>(null);
  const [collapsedIds, setCollapsedIds] = useState<Set<string>>(new Set());
  const [gridReady, setGridReady] = useState(false);
  const [layoutColumns, setLayoutColumns] = useState<Map<string, Map<string, string>>>(new Map());
  const [publishMenuOpen, setPublishMenuOpen] = useState(false);
  const [splitView, setSplitView] = useState(false);
  const [navigationHistory, setNavigationHistory] = useState<string[]>([]);
  const [objectLinks, setObjectLinks] = useState<Link[]>([]);
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
      setSelectedObjectId(id);
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
        if (activePanel && "objectId" in activePanel && activePanel.objectId === id) setActivePanel(null);
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

  const selectedObject = useMemo(
    () => objects.find((o) => o.id === selectedObjectId) ?? null,
    [objects, selectedObjectId],
  );

  const selectedSiblings = useMemo(() => {
    if (!selectedObject) return [];
    return objects.filter((o) => o.parent_id === selectedObject.parent_id);
  }, [objects, selectedObject]);

  const selectedSiblingIndex = useMemo(
    () => (selectedObject ? selectedSiblings.findIndex((o) => o.id === selectedObject.id) : -1),
    [selectedSiblings, selectedObject],
  );

  const canMoveUp = selectedObject != null && selectedSiblingIndex > 0;
  const canMoveDown = selectedObject != null && selectedSiblingIndex >= 0 && selectedSiblingIndex < selectedSiblings.length - 1;
  const canIndent = selectedObject != null && selectedSiblingIndex > 0;
  const canDedent = selectedObject != null && selectedObject.parent_id != null;

  const handleMove = useCallback(
    async (action: "up" | "down" | "indent" | "dedent") => {
      if (!selectedObjectId) return;
      try {
        await api.moveObject(module.id, selectedObjectId, { action });
        fetchObjects();
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to move object");
      }
    },
    [module.id, selectedObjectId, fetchObjects],
  );

  const handleRowDragEnd = useCallback(
    async (event: RowDragEndEvent<ReqObject>) => {
      const draggedData = event.node.data;
      const overData = event.overNode?.data;
      if (!draggedData || !overData || draggedData.id === overData.id) return;

      try {
        await api.moveObject(module.id, draggedData.id, {
          action: "move_to",
          parent_id: overData.parent_id,
          position: overData.position,
        });
        fetchObjects();
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to move object");
      }
    },
    [module.id, fetchObjects],
  );

  // Keyboard shortcuts for reordering (Alt+Arrows) + Enter=add sibling, Tab=add child
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (!selectedObjectId) return;

      // Don't handle shortcuts when editing cells or inside inputs
      const active = document.activeElement;
      const tag = (active?.tagName ?? "").toLowerCase();
      if (tag === "input" || tag === "textarea" || tag === "select") return;
      if (active instanceof HTMLElement && active.isContentEditable) return;
      // Skip when ag-grid cell editor is active
      if (active?.closest(".ag-cell-edit-wrapper")) return;

      if (e.altKey) {
        if (e.key === "ArrowUp") {
          e.preventDefault();
          if (canMoveUp) handleMove("up");
        } else if (e.key === "ArrowDown") {
          e.preventDefault();
          if (canMoveDown) handleMove("down");
        } else if (e.key === "ArrowRight") {
          e.preventDefault();
          if (canIndent) handleMove("indent");
        } else if (e.key === "ArrowLeft") {
          e.preventDefault();
          if (canDedent) handleMove("dedent");
        }
        return;
      }

      // Enter = add sibling (same parent_id, position after selected)
      if (e.key === "Enter" && !e.ctrlKey && !e.shiftKey) {
        const sel = objects.find((o) => o.id === selectedObjectId);
        if (sel) {
          e.preventDefault();
          api.createObject(module.id, {
            parent_id: sel.parent_id ?? undefined,
            position: sel.position + 1,
          }).then(() => fetchObjects()).catch(() => {});
        }
      }

      // Tab = add child (parent_id = selected)
      if (e.key === "Tab" && !e.ctrlKey && !e.shiftKey) {
        e.preventDefault();
        api.createObject(module.id, {
          parent_id: selectedObjectId,
        }).then(() => fetchObjects()).catch(() => {});
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [selectedObjectId, canMoveUp, canMoveDown, canIndent, canDedent, handleMove, objects, module.id, fetchObjects]);

  // Build breadcrumb path for selected object
  const breadcrumbPath = useMemo(() => {
    if (!selectedObjectId) return [];
    const path: ReqObject[] = [];
    let current = objects.find((o) => o.id === selectedObjectId);
    while (current) {
      path.unshift(current);
      current = current.parent_id ? objects.find((o) => o.id === current!.parent_id) : undefined;
    }
    return path;
  }, [selectedObjectId, objects]);

  // Highlight IDs for search results in tree
  const searchHighlightIds = useMemo(() => {
    if (!activeSearch) return undefined;
    return new Set(objects.map((o) => o.id));
  }, [activeSearch, objects]);

  // Batch review: mark all unreviewed objects as reviewed
  const handleBatchReview = useCallback(async () => {
    const unreviewed = objects.filter((o) => !isReviewed(o));
    if (unreviewed.length === 0) return;
    if (!window.confirm(`Mark ${unreviewed.length} unreviewed object(s) as reviewed?`)) return;
    try {
      const results = await Promise.allSettled(
        unreviewed.map((obj) => api.updateObject(module.id, obj.id, { reviewed: true })),
      );
      const failures = results.filter((r) => r.status === "rejected");
      if (failures.length > 0) {
        setError(`Batch review: ${failures.length} of ${unreviewed.length} failed`);
      }
      fetchObjects();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to batch review");
    }
  }, [objects, module.id, fetchObjects]);

  // Fetch links for selected object (for hyperlink navigation)
  useEffect(() => {
    if (!selectedObjectId) {
      setObjectLinks([]);
      return;
    }
    let cancelled = false;
    (async () => {
      try {
        const [srcLinks, tgtLinks] = await Promise.all([
          api.listLinks({ source_object_id: selectedObjectId }),
          api.listLinks({ target_object_id: selectedObjectId }),
        ]);
        if (!cancelled) {
          setObjectLinks([...srcLinks.items, ...tgtLinks.items]);
        }
      } catch {
        if (!cancelled) setObjectLinks([]);
      }
    })();
    return () => { cancelled = true; };
  }, [selectedObjectId]);

  // Navigate to a linked object
  const handleNavigateToObject = useCallback((objectId: string, targetModuleId?: string) => {
    if (selectedObjectId) {
      setNavigationHistory((prev) => [...prev, selectedObjectId]);
    }
    if (targetModuleId && targetModuleId !== module.id) {
      // Cross-module navigation - use URL
      const path = window.location.pathname.replace(/\/m\/[^/]+/, `/m/${targetModuleId}`);
      navigate(path);
    } else {
      setSelectedObjectId(objectId);
      setActivePanel({ type: "history", objectId });
      const gridApi = gridRef.current?.api;
      if (gridApi) {
        const rowNode = gridApi.getRowNode(objectId);
        if (rowNode) {
          gridApi.ensureNodeVisible(rowNode);
          gridApi.setNodesSelected({ nodes: [rowNode], newValue: true });
        }
      }
    }
  }, [selectedObjectId, module.id, navigate]);

  const handleNavigateBack = useCallback(() => {
    setNavigationHistory((prev) => {
      if (prev.length === 0) return prev;
      const newHistory = [...prev];
      const prevId = newHistory.pop()!;
      setSelectedObjectId(prevId);
      setActivePanel({ type: "history", objectId: prevId });
      return newHistory;
    });
  }, []);

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
      { headerName: "Level", field: "level", width: 90, rowDrag: true },
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
        width: 250,
        sortable: false,
        filter: false,
        cellRenderer: (p: ICellRendererParams<ReqObject>) =>
          p.data ? (
            <span style={{ display: "inline-flex", gap: "4px" }}>
              <button
                title="Edit"
                onClick={(e) => {
                  e.stopPropagation();
                  setActivePanel({ type: "detail", objectId: p.data!.id });
                }}
                style={{ padding: "2px 6px", fontSize: "0.8rem" }}
              >
                Edit
              </button>
              <button
                title="References"
                onClick={(e) => {
                  e.stopPropagation();
                  setActivePanel({ type: "references", objectId: p.data!.id });
                }}
                style={{ padding: "2px 6px", fontSize: "0.8rem" }}
              >
                Ref
              </button>
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
              {p.data.reviewed_fingerprint !== p.data.content_fingerprint && (
                <button
                  title="Review Diff"
                  onClick={(e) => {
                    e.stopPropagation();
                    setActivePanel({ type: "reviewDiff", objectId: p.data!.id });
                  }}
                  style={{ padding: "2px 6px", fontSize: "0.8rem" }}
                >
                  Diff
                </button>
              )}
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

  return (
    <div>
      <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
        <h1 style={{ marginTop: 0 }}>{module.name}</h1>
        <div style={{ display: "flex", gap: theme.spacing.sm, alignItems: "center" }}>
          <button
            onClick={() => setActivePanel({ type: "preview" })}
            style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}`, height: "fit-content" }}
          >
            Preview
          </button>
          <div style={{ position: "relative" }}>
            <button
              onClick={() => setPublishMenuOpen((p) => !p)}
              style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}`, height: "fit-content" }}
            >
              Publish &#9662;
            </button>
            {publishMenuOpen && (
              <div
                style={{
                  position: "absolute",
                  top: "100%",
                  right: 0,
                  background: theme.colors.bg,
                  border: `1px solid ${theme.colors.border}`,
                  borderRadius: theme.borderRadius,
                  boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
                  zIndex: 100,
                  minWidth: 140,
                }}
              >
                {PUBLISH_FORMATS.map((fmt) => (
                  <button
                    key={fmt.value}
                    disabled={!fmt.enabled}
                    onClick={() => {
                      window.open(api.getPublishUrl(module.id, fmt.value), "_blank");
                      setPublishMenuOpen(false);
                    }}
                    style={{
                      display: "block",
                      width: "100%",
                      padding: `${theme.spacing.sm} ${theme.spacing.md}`,
                      border: "none",
                      background: "none",
                      textAlign: "left",
                      cursor: fmt.enabled ? "pointer" : "default",
                      color: fmt.enabled ? theme.colors.text : theme.colors.textMuted,
                      fontSize: "0.9rem",
                    }}
                    onMouseEnter={(e) => { if (fmt.enabled) (e.currentTarget.style.background = theme.colors.bgHover); }}
                    onMouseLeave={(e) => { e.currentTarget.style.background = "none"; }}
                  >
                    {fmt.label}
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
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

      {/* Breadcrumb for selected object */}
      {breadcrumbPath.length > 0 && activeTab === "objects" && (
        <div style={{ display: "flex", alignItems: "center", gap: "4px", marginBottom: theme.spacing.sm, fontSize: "0.85rem" }}>
          {navigationHistory.length > 0 && (
            <button
              onClick={handleNavigateBack}
              style={{ padding: "2px 8px", fontSize: "0.8rem", marginRight: theme.spacing.sm }}
            >
              Back
            </button>
          )}
          {breadcrumbPath.map((obj, i) => (
            <span key={obj.id}>
              {i > 0 && <span style={{ color: theme.colors.textMuted, margin: "0 2px" }}>&gt;</span>}
              <span
                style={{
                  cursor: i < breadcrumbPath.length - 1 ? "pointer" : "default",
                  color: i < breadcrumbPath.length - 1 ? theme.colors.primary : theme.colors.text,
                  fontWeight: i === breadcrumbPath.length - 1 ? 600 : 400,
                }}
                onClick={() => {
                  if (i < breadcrumbPath.length - 1) {
                    setSelectedObjectId(obj.id);
                    setActivePanel({ type: "history", objectId: obj.id });
                  }
                }}
              >
                {obj.level} {obj.heading ?? ""}
              </span>
            </span>
          ))}
        </div>
      )}

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
            <button
              onClick={handleBatchReview}
              style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}`, fontSize: "0.85rem" }}
            >
              Review All
            </button>
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

          {/* Reorder toolbar */}
          <div style={{ display: "flex", gap: theme.spacing.sm, marginBottom: "0.75rem", alignItems: "center" }}>
            <button
              disabled={!canMoveUp}
              onClick={() => handleMove("up")}
              style={{ padding: theme.spacing.sm }}
              title="Move Up (Alt+Up)"
            >
              Move Up
            </button>
            <button
              disabled={!canMoveDown}
              onClick={() => handleMove("down")}
              style={{ padding: theme.spacing.sm }}
              title="Move Down (Alt+Down)"
            >
              Move Down
            </button>
            <button
              disabled={!canIndent}
              onClick={() => handleMove("indent")}
              style={{ padding: theme.spacing.sm }}
              title="Indent (Alt+Right)"
            >
              Indent
            </button>
            <button
              disabled={!canDedent}
              onClick={() => handleMove("dedent")}
              style={{ padding: theme.spacing.sm }}
              title="Dedent (Alt+Left)"
            >
              Dedent
            </button>
            {selectedObject && (
              <span style={{ color: theme.colors.textSecondary, fontSize: "0.85rem" }}>
                Selected: {selectedObject.heading ?? selectedObject.id}
              </span>
            )}
          </div>

          {/* Bulk operations toolbar */}
          {(() => {
            const selected = gridRef.current?.api?.getSelectedRows() ?? [];
            if (selected.length === 0) return null;
            return (
              <div style={{ display: "flex", gap: theme.spacing.sm, marginBottom: "0.75rem", alignItems: "center", padding: theme.spacing.sm, background: "#e3f2fd", borderRadius: theme.borderRadius }}>
                <span style={{ fontSize: "0.85rem", fontWeight: 600 }}>
                  {selected.length} selected
                </span>
                <button
                  onClick={async () => {
                    if (!window.confirm(`Delete ${selected.length} selected object(s)?`)) return;
                    await Promise.allSettled(
                      selected.map((obj) => api.deleteObject(module.id, obj.id)),
                    );
                    fetchObjects();
                  }}
                  style={{ padding: "4px 10px", fontSize: "0.85rem" }}
                >
                  Batch Delete
                </button>
                <button
                  onClick={async () => {
                    if (!window.confirm(`Mark ${selected.length} object(s) as reviewed?`)) return;
                    await Promise.allSettled(
                      selected.map((obj) => api.updateObject(module.id, obj.id, { reviewed: true })),
                    );
                    fetchObjects();
                  }}
                  style={{ padding: "4px 10px", fontSize: "0.85rem" }}
                >
                  Batch Review
                </button>
                <select
                  onChange={async (e) => {
                    const cls = e.target.value;
                    if (!cls) return;
                    await Promise.allSettled(
                      selected.map((obj) => api.updateObject(module.id, obj.id, { classification: cls })),
                    );
                    fetchObjects();
                    e.target.value = "";
                  }}
                  style={{ padding: "4px", fontSize: "0.85rem" }}
                >
                  <option value="">Batch Classify...</option>
                  <option value="normative">Normative</option>
                  <option value="informative">Informative</option>
                  <option value="heading">Heading</option>
                </select>
              </div>
            );
          })()}

          <div style={{ display: "flex", gap: theme.spacing.sm, marginBottom: "0.75rem", alignItems: "center" }}>
            <ViewBar moduleId={module.id} gridRef={gridRef} gridReady={gridReady} />
            <button
              onClick={() => setSplitView((p) => !p)}
              style={{ padding: `${theme.spacing.sm} ${theme.spacing.md}`, fontSize: "0.85rem" }}
            >
              {splitView ? "Grid View" : "Split View"}
            </button>
          </div>

          {/* Linked objects for selected object */}
          {selectedObject && objectLinks.length > 0 && (
            <div style={{ display: "flex", gap: theme.spacing.sm, marginBottom: "0.75rem", alignItems: "center", flexWrap: "wrap", fontSize: "0.85rem" }}>
              <span style={{ color: theme.colors.textSecondary, fontWeight: 600 }}>Links:</span>
              {objectLinks.map((link) => {
                const isSource = link.source_object_id === selectedObjectId;
                const linkedId = isSource ? link.target_object_id : link.source_object_id;
                const linkedObj = objects.find((o) => o.id === linkedId);
                return (
                  <button
                    key={link.id}
                    onClick={() => handleNavigateToObject(linkedId, linkedObj?.module_id)}
                    style={{
                      padding: "2px 8px",
                      fontSize: "0.8rem",
                      background: theme.colors.bgCode,
                      border: `1px solid ${theme.colors.borderLight}`,
                      borderRadius: theme.borderRadius,
                      cursor: "pointer",
                    }}
                    title={`${isSource ? "Target" : "Source"}: ${linkedId.slice(0, 8)}...`}
                  >
                    {linkedObj ? (linkedObj.heading ?? linkedObj.level) : linkedId.slice(0, 8) + "..."}
                    {link.suspect && <span style={{ color: theme.colors.suspect, marginLeft: 4 }}>!</span>}
                  </button>
                );
              })}
            </div>
          )}

          {splitView ? (
            <div style={{ display: "flex", height: 500 }}>
              <ObjectTree
                objects={objects}
                selectedId={selectedObjectId}
                onSelect={handleTreeNodeSelect}
                highlightIds={searchHighlightIds}
              />
              <div style={{ flex: 1, overflow: "auto" }}>
                {selectedObject ? (
                  <ObjectDetailPanel
                    moduleId={module.id}
                    objectId={selectedObject.id}
                    attrDefs={attrDefs}
                    objectTypes={objectTypes}
                    inline
                    onClose={() => setSelectedObjectId(null)}
                    onSaved={fetchObjects}
                  />
                ) : (
                  <div style={{ padding: theme.spacing.lg, color: theme.colors.textMuted, textAlign: "center" }}>
                    Select an object from the tree
                  </div>
                )}
              </div>
            </div>
          ) : (
            <div style={{ display: "flex", height: 500 }}>
              <ObjectTree
                objects={objects}
                selectedId={(activePanel && "objectId" in activePanel ? activePanel.objectId : null)}
                onSelect={handleTreeNodeSelect}
                highlightIds={searchHighlightIds}
              />
              <div style={{ flex: 1 }}>
                <AgGridReact<ReqObject>
                  ref={gridRef}
                  theme={themeQuartz}
                  modules={[AllCommunityModule]}
                  rowData={visibleObjects}
                  columnDefs={columnDefs}
                  getRowId={(p) => p.data.id}
                  rowDragManaged={false}
                  rowSelection="multiple"
                  onRowDragEnd={handleRowDragEnd}
                  onCellValueChanged={handleCellValueChanged}
                  onGridReady={() => setGridReady(true)}
                  onRowClicked={(e) => {
                    const target = e.event?.target as HTMLElement | undefined;
                    if (target?.closest("button")) return;
                    if (e.data) {
                      setSelectedObjectId(e.data.id);
                      setActivePanel({ type: "history", objectId: e.data.id });
                    }
                  }}
                />
              </div>
            </div>
          )}

          <ActivityFeed moduleId={module.id} objects={objects} />
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

      {activeTab === "reviews" && (
        <ReviewPanel moduleId={module.id} />
      )}

      {activeTab === "proposals" && (
        <ChangeProposalPanel moduleId={module.id} objects={objects} onObjectsChanged={fetchObjects} />
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
      {activePanel?.type === "detail" && (
        <ObjectDetailPanel
          moduleId={module.id}
          objectId={activePanel.objectId}
          attrDefs={attrDefs}
          objectTypes={objectTypes}
          onClose={() => setActivePanel(null)}
          onSaved={fetchObjects}
        />
      )}
      {activePanel?.type === "references" && (
        <ReferencesPanel
          moduleId={module.id}
          objectId={activePanel.objectId}
          onClose={() => setActivePanel(null)}
          onSaved={fetchObjects}
        />
      )}
      {activePanel?.type === "reviewDiff" && (
        <ReviewDiffPanel
          moduleId={module.id}
          objectId={activePanel.objectId}
          onClose={() => setActivePanel(null)}
        />
      )}
      {activePanel?.type === "preview" && (
        <PublishPreviewPanel
          moduleId={module.id}
          onClose={() => setActivePanel(null)}
        />
      )}
    </div>
  );
}

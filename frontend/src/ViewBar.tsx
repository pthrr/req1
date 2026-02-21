import { type RefObject, useCallback, useEffect, useState } from "react";
import type { AgGridReact } from "ag-grid-react";
import { api, type View } from "./api/client";
import type { ReqObject } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
  gridRef: RefObject<AgGridReact<ReqObject> | null>;
  gridReady: boolean;
}

export function ViewBar({ moduleId, gridRef, gridReady }: Props) {
  const [views, setViews] = useState<View[]>([]);
  const [selectedViewId, setSelectedViewId] = useState<string | null>(null);
  const [saveName, setSaveName] = useState("");
  const [showSaveDialog, setShowSaveDialog] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const fetchViews = useCallback(async () => {
    try {
      const data = await api.listViews(moduleId);
      setViews(data.items);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load views");
    }
  }, [moduleId]);

  const applyView = useCallback(
    (view: View) => {
      const gridApi = gridRef.current?.api;
      if (!gridApi) return;
      if (view.column_config) {
        gridApi.applyColumnState({
          state: view.column_config as Parameters<typeof gridApi.applyColumnState>[0]["state"],
          applyOrder: true,
        });
      }
      if (view.filter_config) {
        gridApi.setFilterModel(view.filter_config as Record<string, unknown>);
      }
      setSelectedViewId(view.id);
    },
    [gridRef],
  );

  useEffect(() => {
    if (!gridReady) return;
    fetchViews().then(() => {
      // Auto-apply handled after views load
    });
  }, [fetchViews, gridReady]);

  // Auto-apply default view when views load
  useEffect(() => {
    if (!gridReady || views.length === 0) return;
    const defaultView = views.find((v) => v.is_default);
    if (defaultView && selectedViewId == null) {
      applyView(defaultView);
    }
  }, [views, gridReady, applyView, selectedViewId]);

  const handleLoad = (viewId: string) => {
    const view = views.find((v) => v.id === viewId);
    if (view) applyView(view);
  };

  const handleSave = async () => {
    const gridApi = gridRef.current?.api;
    if (!gridApi || !saveName.trim()) return;
    try {
      await api.createView(moduleId, {
        name: saveName.trim(),
        column_config: gridApi.getColumnState(),
        filter_config: gridApi.getFilterModel(),
        sort_config: [],
      });
      setSaveName("");
      setShowSaveDialog(false);
      await fetchViews();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to save view");
    }
  };

  const handleUpdate = async () => {
    if (!selectedViewId) return;
    const gridApi = gridRef.current?.api;
    if (!gridApi) return;
    try {
      await api.updateView(moduleId, selectedViewId, {
        column_config: gridApi.getColumnState(),
        filter_config: gridApi.getFilterModel(),
      });
      await fetchViews();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update view");
    }
  };

  const handleSetDefault = async () => {
    if (!selectedViewId) return;
    try {
      // Unset previous default
      const prevDefault = views.find((v) => v.is_default && v.id !== selectedViewId);
      if (prevDefault) {
        await api.updateView(moduleId, prevDefault.id, { is_default: false });
      }
      await api.updateView(moduleId, selectedViewId, { is_default: true });
      await fetchViews();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to set default");
    }
  };

  const handleDelete = async () => {
    if (!selectedViewId) return;
    try {
      await api.deleteView(moduleId, selectedViewId);
      setSelectedViewId(null);
      await fetchViews();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete view");
    }
  };

  return (
    <div style={{ marginBottom: theme.spacing.sm }}>
      {error && (
        <div style={{ color: theme.colors.error, marginBottom: theme.spacing.xs, fontSize: "0.85rem" }}>
          {error}
        </div>
      )}
      <div style={{ display: "flex", gap: theme.spacing.sm, alignItems: "center", flexWrap: "wrap" }}>
        <label style={{ fontSize: "0.85rem", fontWeight: 600 }}>View:</label>
        <select
          value={selectedViewId ?? ""}
          onChange={(e) => {
            if (e.target.value) handleLoad(e.target.value);
            else setSelectedViewId(null);
          }}
          style={{ padding: "0.3rem" }}
        >
          <option value="">(none)</option>
          {views.map((v) => (
            <option key={v.id} value={v.id}>
              {v.name}
              {v.is_default ? " *" : ""}
            </option>
          ))}
        </select>

        {selectedViewId && (
          <>
            <button onClick={handleUpdate} style={{ padding: "0.3rem 0.6rem", fontSize: "0.85rem" }}>
              Update
            </button>
            <button onClick={handleSetDefault} style={{ padding: "0.3rem 0.6rem", fontSize: "0.85rem" }}>
              Set Default
            </button>
            <button onClick={handleDelete} style={{ padding: "0.3rem 0.6rem", fontSize: "0.85rem" }}>
              Delete
            </button>
          </>
        )}

        {showSaveDialog ? (
          <span style={{ display: "inline-flex", gap: theme.spacing.xs, alignItems: "center" }}>
            <input
              type="text"
              value={saveName}
              onChange={(e) => setSaveName(e.target.value)}
              placeholder="View name"
              style={{ padding: "0.3rem", width: "140px" }}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleSave();
              }}
            />
            <button onClick={handleSave} style={{ padding: "0.3rem 0.6rem", fontSize: "0.85rem" }}>
              Save
            </button>
            <button
              onClick={() => {
                setShowSaveDialog(false);
                setSaveName("");
              }}
              style={{ padding: "0.3rem 0.6rem", fontSize: "0.85rem" }}
            >
              Cancel
            </button>
          </span>
        ) : (
          <button
            onClick={() => setShowSaveDialog(true)}
            style={{ padding: "0.3rem 0.6rem", fontSize: "0.85rem" }}
          >
            Save As...
          </button>
        )}
      </div>
    </div>
  );
}

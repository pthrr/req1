import { useEffect, useState } from "react";
import {
  api,
  type LinkType,
  type MatrixCell,
  type MatrixObject,
  type Module,
  type Project,
} from "./api/client";

export function TraceabilityMatrix({
  project,
}: {
  project: Project;
}) {
  const [modules, setModules] = useState<Module[]>([]);
  const [linkTypes, setLinkTypes] = useState<LinkType[]>([]);
  const [sourceModuleId, setSourceModuleId] = useState("");
  const [targetModuleId, setTargetModuleId] = useState("");
  const [linkTypeId, setLinkTypeId] = useState("");
  const [sourceObjects, setSourceObjects] = useState<MatrixObject[]>([]);
  const [targetObjects, setTargetObjects] = useState<MatrixObject[]>([]);
  const [cellMap, setCellMap] = useState<Map<string, MatrixCell>>(new Map());
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api
      .listModules({ project_id: project.id })
      .then((data) => setModules(data.items))
      .catch((err) =>
        setError(err instanceof Error ? err.message : "Failed to load modules"),
      );
    api
      .listLinkTypes()
      .then(setLinkTypes)
      .catch((err) =>
        setError(
          err instanceof Error ? err.message : "Failed to load link types",
        ),
      );
  }, [project.id]);

  useEffect(() => {
    if (!sourceModuleId || !targetModuleId) {
      setSourceObjects([]);
      setTargetObjects([]);
      setCellMap(new Map());
      return;
    }

    setLoading(true);
    setError(null);
    api
      .getTraceabilityMatrix(
        sourceModuleId,
        targetModuleId,
        linkTypeId || undefined,
      )
      .then((resp) => {
        setSourceObjects(resp.source_objects);
        setTargetObjects(resp.target_objects);
        const map = new Map<string, MatrixCell>();
        for (const cell of resp.cells) {
          map.set(`${cell.source_id}:${cell.target_id}`, cell);
        }
        setCellMap(map);
      })
      .catch((err) =>
        setError(
          err instanceof Error ? err.message : "Failed to load matrix",
        ),
      )
      .finally(() => setLoading(false));
  }, [sourceModuleId, targetModuleId, linkTypeId]);

  return (
    <>
      <h1 style={{ marginTop: 0 }}>Traceability Matrix</h1>

      {error && (
        <div style={{ color: "red", marginBottom: "1rem" }}>{error}</div>
      )}

      <div style={{ display: "flex", gap: "1rem", marginBottom: "1.5rem", flexWrap: "wrap" }}>
        <label>
          Source module:{" "}
          <select
            value={sourceModuleId}
            onChange={(e) => setSourceModuleId(e.target.value)}
          >
            <option value="">-- select --</option>
            {modules.map((m) => (
              <option key={m.id} value={m.id}>
                {m.name}
              </option>
            ))}
          </select>
        </label>

        <label>
          Target module:{" "}
          <select
            value={targetModuleId}
            onChange={(e) => setTargetModuleId(e.target.value)}
          >
            <option value="">-- select --</option>
            {modules.map((m) => (
              <option key={m.id} value={m.id}>
                {m.name}
              </option>
            ))}
          </select>
        </label>

        <label>
          Link type:{" "}
          <select
            value={linkTypeId}
            onChange={(e) => setLinkTypeId(e.target.value)}
          >
            <option value="">All</option>
            {linkTypes.map((lt) => (
              <option key={lt.id} value={lt.id}>
                {lt.name}
              </option>
            ))}
          </select>
        </label>
      </div>

      {loading && <p>Loading...</p>}

      {!loading &&
        sourceObjects.length > 0 &&
        targetObjects.length > 0 && (
          <div style={{ overflowX: "auto" }}>
            <table
              style={{
                borderCollapse: "collapse",
                fontSize: "0.85rem",
              }}
            >
              <thead>
                <tr>
                  <th
                    style={{
                      border: "1px solid #ccc",
                      padding: "0.3rem 0.5rem",
                      background: "#f5f5f5",
                    }}
                  >
                    Source \ Target
                  </th>
                  {targetObjects.map((t) => (
                    <th
                      key={t.id}
                      style={{
                        border: "1px solid #ccc",
                        padding: "0.3rem 0.5rem",
                        background: "#f5f5f5",
                        maxWidth: 120,
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                      }}
                      title={t.heading ?? `#${t.position}`}
                    >
                      {t.heading ?? `#${t.position}`}
                    </th>
                  ))}
                </tr>
              </thead>
              <tbody>
                {sourceObjects.map((s) => (
                  <tr key={s.id}>
                    <td
                      style={{
                        border: "1px solid #ccc",
                        padding: "0.3rem 0.5rem",
                        background: "#f5f5f5",
                        fontWeight: "bold",
                        whiteSpace: "nowrap",
                      }}
                    >
                      {s.heading ?? `#${s.position}`}
                    </td>
                    {targetObjects.map((t) => {
                      const cell = cellMap.get(`${s.id}:${t.id}`);
                      return (
                        <td
                          key={t.id}
                          style={{
                            border: "1px solid #ccc",
                            padding: "0.3rem 0.5rem",
                            textAlign: "center",
                            background: cell
                              ? cell.suspect
                                ? "#fff3e0"
                                : "#e8f5e9"
                              : undefined,
                          }}
                          title={
                            cell
                              ? cell.suspect
                                ? "Suspect link"
                                : "Linked"
                              : "No link"
                          }
                        >
                          {cell
                            ? cell.suspect
                              ? <span style={{ color: "#e65100", fontWeight: "bold" }}>S</span>
                              : <span style={{ color: "#2e7d32" }}>&#10003;</span>
                            : ""}
                        </td>
                      );
                    })}
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

      {!loading &&
        sourceModuleId &&
        targetModuleId &&
        sourceObjects.length === 0 &&
        targetObjects.length === 0 && (
          <p style={{ color: "#999" }}>
            No objects found in the selected modules.
          </p>
        )}
    </>
  );
}

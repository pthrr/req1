import { useCallback, useEffect, useState } from "react";
import {
  api,
  type Baseline,
  type BaselineDiff,
  type BaselineSet,
  type BaselineWithEntries,
} from "./api/client";
import { InlineDiff, AttributesDiff } from "./DiffView";

interface Props {
  moduleId: string;
}

const cellStyle = { padding: "0.25rem", borderBottom: "1px solid #eee" };
const thStyle = {
  textAlign: "left" as const,
  borderBottom: "1px solid #ccc",
  padding: "0.25rem",
};

export function BaselinePanel({ moduleId }: Props) {
  const [baselines, setBaselines] = useState<Baseline[]>([]);
  const [detail, setDetail] = useState<BaselineWithEntries | null>(null);
  const [diff, setDiff] = useState<BaselineDiff | null>(null);
  const [diffA, setDiffA] = useState("");
  const [diffB, setDiffB] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [expandedModified, setExpandedModified] = useState<Set<string>>(new Set());
  const [newName, setNewName] = useState("");
  const [baselineSets, setBaselineSets] = useState<BaselineSet[]>([]);
  const [selectedSetId, setSelectedSetId] = useState("");
  const [filterSetId, setFilterSetId] = useState("");
  const [newSetName, setNewSetName] = useState("");
  const [newSetVersion, setNewSetVersion] = useState("");
  const [showSetForm, setShowSetForm] = useState(false);
  const [crossModuleMode, setCrossModuleMode] = useState(false);
  const [allBaselines, setAllBaselines] = useState<Baseline[]>([]);

  const fetchBaselines = useCallback(async () => {
    try {
      const data = await api.listBaselines(moduleId);
      setBaselines(data.items);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load baselines");
    }
  }, [moduleId]);

  const fetchAllBaselines = useCallback(async () => {
    try {
      const data = await api.listAllBaselines(500);
      setAllBaselines(data.items);
    } catch {
      // Non-critical
    }
  }, []);

  const fetchBaselineSets = useCallback(async () => {
    try {
      const data = await api.listBaselineSets();
      setBaselineSets(data.items);
    } catch {
      // Non-critical
    }
  }, []);

  useEffect(() => {
    fetchBaselines();
    fetchBaselineSets();
  }, [fetchBaselines, fetchBaselineSets]);

  useEffect(() => {
    if (crossModuleMode) fetchAllBaselines();
  }, [crossModuleMode, fetchAllBaselines]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newName.trim()) return;
    try {
      await api.createBaseline(moduleId, {
        name: newName.trim(),
        baseline_set_id: selectedSetId || undefined,
      });
      setNewName("");
      setSelectedSetId("");
      fetchBaselines();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create baseline");
    }
  };

  const handleCreateSet = async () => {
    if (!newSetName.trim() || !newSetVersion.trim()) return;
    try {
      await api.createBaselineSet({ name: newSetName.trim(), version: newSetVersion.trim() });
      setNewSetName("");
      setNewSetVersion("");
      setShowSetForm(false);
      fetchBaselineSets();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create baseline set");
    }
  };

  const handleDeleteSet = async (id: string) => {
    try {
      await api.deleteBaselineSet(id);
      fetchBaselineSets();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete baseline set");
    }
  };

  const filteredBaselines = filterSetId
    ? baselines.filter((bl) => bl.baseline_set_id === filterSetId)
    : baselines;

  const handleView = async (id: string) => {
    try {
      const data = await api.getBaseline(moduleId, id);
      setDetail(data);
      setDiff(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load baseline");
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.deleteBaseline(moduleId, id);
      if (detail?.id === id) setDetail(null);
      fetchBaselines();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete baseline");
    }
  };

  const handleDiff = async () => {
    if (!diffA || !diffB || diffA === diffB) return;
    try {
      const data = crossModuleMode
        ? await api.diffBaselinesGlobal(diffA, diffB)
        : await api.diffBaselines(moduleId, diffA, diffB);
      setDiff(data);
      setDetail(null);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to diff baselines");
    }
  };

  return (
    <div>
      {error && <div style={{ color: "red", marginBottom: "0.5rem" }}>{error}</div>}

      {/* Baseline Sets Section */}
      <div style={{ marginBottom: "1rem", padding: "0.5rem", background: "#f8f9fa", borderRadius: 4 }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "0.5rem" }}>
          <h4 style={{ margin: 0 }}>Baseline Sets</h4>
          <button onClick={() => setShowSetForm((p) => !p)} style={{ padding: "0.25rem 0.5rem", fontSize: "0.85rem" }}>
            {showSetForm ? "Cancel" : "New Set"}
          </button>
        </div>

        {showSetForm && (
          <div style={{ display: "flex", gap: "0.5rem", marginBottom: "0.5rem" }}>
            <input
              type="text"
              value={newSetName}
              onChange={(e) => setNewSetName(e.target.value)}
              placeholder="Set name"
              style={{ padding: "0.25rem", flex: 1 }}
            />
            <input
              type="text"
              value={newSetVersion}
              onChange={(e) => setNewSetVersion(e.target.value)}
              placeholder="Version"
              style={{ padding: "0.25rem", width: 80 }}
            />
            <button onClick={handleCreateSet} style={{ padding: "0.25rem 0.5rem" }}>
              Create
            </button>
          </div>
        )}

        <div style={{ display: "flex", gap: "0.5rem", flexWrap: "wrap" }}>
          <button
            onClick={() => setFilterSetId("")}
            style={{
              padding: "2px 8px",
              fontSize: "0.8rem",
              background: !filterSetId ? "#e3f2fd" : "transparent",
              border: "1px solid #ddd",
              borderRadius: 3,
              cursor: "pointer",
            }}
          >
            All
          </button>
          {baselineSets.map((s) => (
            <span key={s.id} style={{ display: "inline-flex", alignItems: "center", gap: 2 }}>
              <button
                onClick={() => setFilterSetId(s.id)}
                style={{
                  padding: "2px 8px",
                  fontSize: "0.8rem",
                  background: filterSetId === s.id ? "#e3f2fd" : "transparent",
                  border: "1px solid #ddd",
                  borderRadius: 3,
                  cursor: "pointer",
                }}
              >
                {s.name} v{s.version}
              </button>
              <button
                onClick={() => handleDeleteSet(s.id)}
                style={{ padding: "1px 4px", fontSize: "0.7rem", border: "none", background: "none", cursor: "pointer", color: "#d32f2f" }}
              >
                x
              </button>
            </span>
          ))}
        </div>
      </div>

      <form
        onSubmit={handleCreate}
        style={{ display: "flex", gap: "0.5rem", marginBottom: "1rem" }}
      >
        <input
          type="text"
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          placeholder="Baseline name"
          style={{ padding: "0.25rem", flex: 1 }}
        />
        {baselineSets.length > 0 && (
          <select
            value={selectedSetId}
            onChange={(e) => setSelectedSetId(e.target.value)}
            style={{ padding: "0.25rem" }}
          >
            <option value="">(no set)</option>
            {baselineSets.map((s) => (
              <option key={s.id} value={s.id}>{s.name} v{s.version}</option>
            ))}
          </select>
        )}
        <button type="submit" style={{ padding: "0.25rem 0.75rem" }}>
          Create Baseline
        </button>
      </form>

      <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "0.9rem" }}>
        <thead>
          <tr>
            <th style={thStyle}>Name</th>
            <th style={thStyle}>Set</th>
            <th style={thStyle}>Created</th>
            <th style={thStyle}>Locked</th>
            <th style={{ borderBottom: "1px solid #ccc", padding: "0.25rem" }} />
          </tr>
        </thead>
        <tbody>
          {filteredBaselines.map((bl) => {
            const set = baselineSets.find((s) => s.id === bl.baseline_set_id);
            return (
              <tr key={bl.id}>
                <td style={cellStyle}>{bl.name}</td>
                <td style={cellStyle}>{set ? `${set.name} v${set.version}` : "-"}</td>
                <td style={cellStyle}>{new Date(bl.created_at).toLocaleString()}</td>
                <td style={cellStyle}>{bl.locked ? "Yes" : "No"}</td>
                <td style={cellStyle}>
                  <button onClick={() => handleView(bl.id)} style={{ marginRight: "0.25rem" }}>
                    View
                  </button>
                  <button onClick={() => handleDelete(bl.id)}>Delete</button>
                </td>
              </tr>
            );
          })}
          {filteredBaselines.length === 0 && (
            <tr>
              <td colSpan={5} style={{ padding: "0.5rem", textAlign: "center", color: "#999" }}>
                No baselines yet.
              </td>
            </tr>
          )}
        </tbody>
      </table>

      {(() => {
        const compareBaselines = crossModuleMode ? allBaselines : filteredBaselines;
        if (compareBaselines.length < 2) return null;
        return (
          <div
            style={{
              display: "flex",
              gap: "0.5rem",
              alignItems: "center",
              marginTop: "0.75rem",
              flexWrap: "wrap",
            }}
          >
            <span style={{ fontSize: "0.9rem" }}>Compare:</span>
            <button
              onClick={() => {
                setCrossModuleMode((p) => !p);
                setDiffA("");
                setDiffB("");
                setDiff(null);
              }}
              style={{
                padding: "2px 8px",
                fontSize: "0.8rem",
                background: crossModuleMode ? "#e3f2fd" : "transparent",
                border: "1px solid #ddd",
                borderRadius: 3,
                cursor: "pointer",
              }}
            >
              {crossModuleMode ? "Cross-Module" : "This Module"}
            </button>
            <select value={diffA} onChange={(e) => setDiffA(e.target.value)}>
              <option value="">Baseline A</option>
              {compareBaselines.map((bl) => (
                <option key={bl.id} value={bl.id}>
                  {bl.name}{crossModuleMode ? ` (${bl.module_id.slice(0, 8)}...)` : ""}
                </option>
              ))}
            </select>
            <span>vs</span>
            <select value={diffB} onChange={(e) => setDiffB(e.target.value)}>
              <option value="">Baseline B</option>
              {compareBaselines.map((bl) => (
                <option key={bl.id} value={bl.id}>
                  {bl.name}{crossModuleMode ? ` (${bl.module_id.slice(0, 8)}...)` : ""}
                </option>
              ))}
            </select>
            <button onClick={handleDiff} disabled={!diffA || !diffB || diffA === diffB}>
              Diff
            </button>
          </div>
        );
      })()}

      {detail && (
        <div style={{ marginTop: "1rem" }}>
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              marginBottom: "0.5rem",
            }}
          >
            <h4 style={{ margin: 0 }}>Baseline: {detail.name}</h4>
            <button onClick={() => setDetail(null)}>Close</button>
          </div>
          <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "0.9rem" }}>
            <thead>
              <tr>
                <th style={thStyle}>Object ID</th>
                <th style={thStyle}>Version</th>
              </tr>
            </thead>
            <tbody>
              {detail.entries.map((entry) => (
                <tr key={entry.object_id}>
                  <td style={cellStyle}>{entry.object_id.slice(0, 12)}...</td>
                  <td style={cellStyle}>{entry.version}</td>
                </tr>
              ))}
              {detail.entries.length === 0 && (
                <tr>
                  <td colSpan={2} style={{ padding: "0.5rem", textAlign: "center", color: "#999" }}>
                    No entries in this baseline.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      )}

      {diff && (
        <div style={{ marginTop: "1rem" }}>
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              marginBottom: "0.5rem",
            }}
          >
            <h4 style={{ margin: 0 }}>Baseline Diff</h4>
            <button onClick={() => setDiff(null)}>Close</button>
          </div>

          {diff.added.length > 0 && (
            <>
              <h5 style={{ marginBottom: "0.25rem", color: "green" }}>
                Added ({diff.added.length})
              </h5>
              <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "0.85rem" }}>
                <thead>
                  <tr>
                    <th style={thStyle}>Object ID</th>
                    <th style={thStyle}>Version</th>
                    <th style={thStyle}>Heading</th>
                  </tr>
                </thead>
                <tbody>
                  {diff.added.map((e) => (
                    <tr key={e.object_id} style={{ background: "#e6ffe6" }}>
                      <td style={cellStyle}>{e.object_id.slice(0, 12)}...</td>
                      <td style={cellStyle}>{e.version}</td>
                      <td style={cellStyle}>{e.heading ?? "-"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}

          {diff.removed.length > 0 && (
            <>
              <h5 style={{ marginBottom: "0.25rem", color: "red" }}>
                Removed ({diff.removed.length})
              </h5>
              <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "0.85rem" }}>
                <thead>
                  <tr>
                    <th style={thStyle}>Object ID</th>
                    <th style={thStyle}>Version</th>
                    <th style={thStyle}>Heading</th>
                  </tr>
                </thead>
                <tbody>
                  {diff.removed.map((e) => (
                    <tr key={e.object_id} style={{ background: "#ffe6e6" }}>
                      <td style={cellStyle}>{e.object_id.slice(0, 12)}...</td>
                      <td style={cellStyle}>{e.version}</td>
                      <td style={cellStyle}>{e.heading ?? "-"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}

          {diff.modified.length > 0 && (
            <>
              <h5 style={{ marginBottom: "0.25rem", color: "orange" }}>
                Modified ({diff.modified.length})
              </h5>
              <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "0.85rem" }}>
                <thead>
                  <tr>
                    <th style={thStyle} />
                    <th style={thStyle}>Object ID</th>
                    <th style={thStyle}>Ver A</th>
                    <th style={thStyle}>Ver B</th>
                    <th style={thStyle}>Heading</th>
                  </tr>
                </thead>
                <tbody>
                  {diff.modified.map((e) => {
                    const isExpanded = expandedModified.has(e.object_id);
                    return (
                      <tr key={e.object_id} style={{ verticalAlign: "top" }}>
                        <td colSpan={5} style={{ padding: 0 }}>
                          <div
                            style={{
                              display: "flex",
                              background: "#fff5e6",
                              cursor: "pointer",
                              padding: "0.25rem",
                              borderBottom: "1px solid #eee",
                            }}
                            onClick={() => {
                              setExpandedModified((prev) => {
                                const next = new Set(prev);
                                if (next.has(e.object_id)) {
                                  next.delete(e.object_id);
                                } else {
                                  next.add(e.object_id);
                                }
                                return next;
                              });
                            }}
                          >
                            <span
                              style={{
                                width: 20,
                                textAlign: "center",
                                fontSize: "0.7em",
                                transition: "transform 0.15s",
                                display: "inline-block",
                                transform: isExpanded ? "rotate(0deg)" : "rotate(-90deg)",
                              }}
                            >
                              {"\u25BC"}
                            </span>
                            <span style={{ width: "25%", overflow: "hidden", textOverflow: "ellipsis" }}>
                              {e.object_id.slice(0, 12)}...
                            </span>
                            <span style={{ width: "10%" }}>v{e.version_a}</span>
                            <span style={{ width: "10%" }}>v{e.version_b}</span>
                            <span style={{ flex: 1 }}>
                              {e.heading_b ?? e.heading_a ?? "-"}
                            </span>
                          </div>
                          {isExpanded && (
                            <div style={{ padding: "0.5rem 0.5rem 0.5rem 28px", background: "#fffdf5" }}>
                              <InlineDiff label="Heading" textA={e.heading_a} textB={e.heading_b} />
                              <InlineDiff label="Body" textA={e.body_a} textB={e.body_b} />
                              <AttributesDiff attrsA={e.attributes_a} attrsB={e.attributes_b} />
                            </div>
                          )}
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </>
          )}

          {diff.added.length === 0 &&
            diff.removed.length === 0 &&
            diff.modified.length === 0 && (
              <p style={{ color: "#999", fontSize: "0.9rem" }}>
                Baselines are identical.
              </p>
            )}
        </div>
      )}
    </div>
  );
}

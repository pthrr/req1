import { useCallback, useEffect, useState } from "react";
import {
  api,
  type Link,
  type LinkType,
  type ReqObject,
} from "./api/client";

interface Props {
  moduleId: string;
  objects: ReqObject[];
}

export function LinkPanel({ moduleId, objects }: Props) {
  const [links, setLinks] = useState<Link[]>([]);
  const [linkTypes, setLinkTypes] = useState<LinkType[]>([]);
  const [error, setError] = useState<string | null>(null);

  const [sourceId, setSourceId] = useState("");
  const [targetId, setTargetId] = useState("");
  const [linkTypeId, setLinkTypeId] = useState("");

  const fetchLinks = useCallback(async () => {
    try {
      const res = await api.listLinks({ module_id: moduleId });
      setLinks(res.items);
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load links");
    }
  }, [moduleId]);

  const fetchLinkTypes = useCallback(async () => {
    try {
      const types = await api.listLinkTypes();
      setLinkTypes(types);
      if (types.length > 0 && !linkTypeId) setLinkTypeId(types[0].id);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load link types");
    }
  }, []);

  useEffect(() => {
    fetchLinks();
    fetchLinkTypes();
  }, [fetchLinks, fetchLinkTypes]);

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!sourceId || !targetId || !linkTypeId) return;
    try {
      await api.createLink({
        source_object_id: sourceId,
        target_object_id: targetId,
        link_type_id: linkTypeId,
      });
      setSourceId("");
      setTargetId("");
      fetchLinks();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create link");
    }
  };

  const handleResolve = async (id: string) => {
    try {
      await api.updateLink(id, { suspect: false });
      fetchLinks();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to resolve link");
    }
  };

  const handleDelete = async (id: string) => {
    try {
      await api.deleteLink(id);
      fetchLinks();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete link");
    }
  };

  const objectLabel = (id: string) => {
    const obj = objects.find((o) => o.id === id);
    return obj?.heading ?? id.slice(0, 8) + "...";
  };

  const linkTypeName = (id: string) =>
    linkTypes.find((lt) => lt.id === id)?.name ?? id.slice(0, 8);

  return (
    <div>
      {error && <div style={{ color: "red", marginBottom: "0.5rem" }}>{error}</div>}

      <form
        onSubmit={handleCreate}
        style={{ display: "flex", gap: "0.5rem", marginBottom: "1rem", flexWrap: "wrap" }}
      >
        <select value={sourceId} onChange={(e) => setSourceId(e.target.value)}>
          <option value="">Source...</option>
          {objects.map((o) => (
            <option key={o.id} value={o.id}>
              {o.heading ?? o.id.slice(0, 8)}
            </option>
          ))}
        </select>

        <select value={linkTypeId} onChange={(e) => setLinkTypeId(e.target.value)}>
          {linkTypes.map((lt) => (
            <option key={lt.id} value={lt.id}>
              {lt.name}
            </option>
          ))}
        </select>

        <select value={targetId} onChange={(e) => setTargetId(e.target.value)}>
          <option value="">Target...</option>
          {objects.map((o) => (
            <option key={o.id} value={o.id}>
              {o.heading ?? o.id.slice(0, 8)}
            </option>
          ))}
        </select>

        <button type="submit" style={{ padding: "0.25rem 0.75rem" }}>
          Create Link
        </button>
      </form>

      <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "0.9rem" }}>
        <thead>
          <tr>
            <th style={{ textAlign: "left", borderBottom: "1px solid #ccc", padding: "0.25rem" }}>
              Source
            </th>
            <th style={{ textAlign: "left", borderBottom: "1px solid #ccc", padding: "0.25rem" }}>
              Type
            </th>
            <th style={{ textAlign: "left", borderBottom: "1px solid #ccc", padding: "0.25rem" }}>
              Target
            </th>
            <th style={{ textAlign: "left", borderBottom: "1px solid #ccc", padding: "0.25rem" }}>
              Status
            </th>
            <th style={{ borderBottom: "1px solid #ccc", padding: "0.25rem" }} />
          </tr>
        </thead>
        <tbody>
          {links.map((lnk) => (
            <tr key={lnk.id}>
              <td style={{ padding: "0.25rem", borderBottom: "1px solid #eee" }}>
                {objectLabel(lnk.source_object_id)}
              </td>
              <td style={{ padding: "0.25rem", borderBottom: "1px solid #eee" }}>
                {linkTypeName(lnk.link_type_id)}
              </td>
              <td style={{ padding: "0.25rem", borderBottom: "1px solid #eee" }}>
                {objectLabel(lnk.target_object_id)}
              </td>
              <td style={{ padding: "0.25rem", borderBottom: "1px solid #eee" }}>
                {lnk.suspect ? (
                  <span style={{ color: "orange", fontWeight: "bold" }}>
                    SUSPECT{" "}
                    <button
                      onClick={() => handleResolve(lnk.id)}
                      style={{ fontSize: "0.8rem" }}
                    >
                      Resolve
                    </button>
                  </span>
                ) : (
                  <span style={{ color: "green" }}>OK</span>
                )}
              </td>
              <td style={{ padding: "0.25rem", borderBottom: "1px solid #eee" }}>
                <button onClick={() => handleDelete(lnk.id)}>Delete</button>
              </td>
            </tr>
          ))}
          {links.length === 0 && (
            <tr>
              <td colSpan={5} style={{ padding: "0.5rem", textAlign: "center", color: "#999" }}>
                No links.
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}

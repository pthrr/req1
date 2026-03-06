import { useCallback, useRef, useState } from "react";
import { api, type ReqObject } from "./api/client";
import { useNavigate } from "react-router";
import { useTheme } from "./ThemeContext";

interface SearchResult extends ReqObject {
  module_name: string;
  project_id: string;
  workspace_id: string;
}

export function GlobalSearch() {
  const { theme } = useTheme();
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [open, setOpen] = useState(false);
  const [loading, setLoading] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout>>(null);

  const doSearch = useCallback(async (q: string) => {
    if (!q.trim()) {
      setResults([]);
      setOpen(false);
      return;
    }
    setLoading(true);
    try {
      const data = await api.searchGlobal(q.trim(), 20);
      setResults(data.items as SearchResult[]);
      setOpen(true);
    } catch {
      setResults([]);
    } finally {
      setLoading(false);
    }
  }, []);

  const handleChange = (value: string) => {
    setQuery(value);
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => doSearch(value), 300);
  };

  const handleSelect = (result: SearchResult) => {
    setOpen(false);
    setQuery("");
    setResults([]);
    navigate(`/w/${result.workspace_id}/p/${result.project_id}/m/${result.module_id}`);
  };

  return (
    <div style={{ position: "relative" }}>
      <input
        type="text"
        value={query}
        onChange={(e) => handleChange(e.target.value)}
        onFocus={() => { if (results.length > 0) setOpen(true); }}
        onBlur={() => setTimeout(() => setOpen(false), 200)}
        placeholder="Search all modules..."
        style={{
          padding: "4px 10px",
          fontSize: "0.85rem",
          width: 220,
          border: `1px solid ${theme.colors.border}`,
          borderRadius: theme.borderRadius,
          background: theme.colors.bg,
          color: theme.colors.text,
        }}
      />
      {loading && (
        <span style={{ position: "absolute", right: 8, top: 6, fontSize: "0.75rem", color: theme.colors.textMuted }}>
          ...
        </span>
      )}
      {open && results.length > 0 && (
        <div
          style={{
            position: "absolute",
            top: "100%",
            left: 0,
            right: 0,
            background: theme.colors.bg,
            border: `1px solid ${theme.colors.border}`,
            borderRadius: theme.borderRadius,
            boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
            zIndex: 200,
            maxHeight: 400,
            overflow: "auto",
            minWidth: 350,
          }}
        >
          {results.map((r) => (
            <div
              key={r.id}
              onClick={() => handleSelect(r)}
              onMouseDown={(e) => e.preventDefault()}
              style={{
                padding: "8px 12px",
                cursor: "pointer",
                borderBottom: `1px solid ${theme.colors.borderLight}`,
                fontSize: "0.85rem",
              }}
              onMouseEnter={(e) => { e.currentTarget.style.background = theme.colors.bgHover; }}
              onMouseLeave={(e) => { e.currentTarget.style.background = "transparent"; }}
            >
              <div style={{ fontWeight: 500 }}>
                {r.level} {r.heading ?? "(no heading)"}
              </div>
              <div style={{ fontSize: "0.8rem", color: theme.colors.textMuted }}>
                {r.module_name} &middot; v{r.current_version}
              </div>
              {r.body && (
                <div style={{ fontSize: "0.8rem", color: theme.colors.textSecondary, marginTop: 2, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap", maxWidth: 320 }}>
                  {r.body.slice(0, 100)}
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

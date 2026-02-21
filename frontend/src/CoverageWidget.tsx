import { useEffect, useState } from "react";
import { api, type CoverageResponse } from "./api/client";
import { theme } from "./theme";

interface Props {
  moduleId: string;
}

function ProgressBar({ label, value, total, pct }: { label: string; value: number; total: number; pct: number }) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: theme.spacing.sm, fontSize: "0.85rem" }}>
      <span style={{ width: 140, flexShrink: 0, fontWeight: 500 }}>{label}</span>
      <div
        style={{
          flex: 1,
          height: 16,
          background: theme.colors.borderLight,
          borderRadius: 8,
          overflow: "hidden",
          position: "relative",
        }}
      >
        <div
          style={{
            width: `${Math.min(pct, 100)}%`,
            height: "100%",
            background: pct >= 80 ? "#4caf50" : pct >= 50 ? "#ff9800" : "#f44336",
            borderRadius: 8,
            transition: "width 0.3s ease",
          }}
        />
      </div>
      <span style={{ width: 120, textAlign: "right", color: theme.colors.textSecondary, flexShrink: 0 }}>
        {pct.toFixed(1)}% ({value}/{total})
      </span>
    </div>
  );
}

export function CoverageWidget({ moduleId }: Props) {
  const [data, setData] = useState<CoverageResponse | null>(null);

  useEffect(() => {
    let cancelled = false;
    api.getCoverage(moduleId).then((r) => {
      if (!cancelled) setData(r);
    }).catch(() => {});
    return () => { cancelled = true; };
  }, [moduleId]);

  if (!data || data.total_objects === 0) return null;

  return (
    <div
      style={{
        marginBottom: theme.spacing.md,
        padding: theme.spacing.sm,
        border: `1px solid ${theme.colors.borderLight}`,
        borderRadius: theme.borderRadius,
        display: "flex",
        flexDirection: "column",
        gap: theme.spacing.xs,
      }}
    >
      <div style={{ fontWeight: 600, fontSize: "0.85rem", marginBottom: 2 }}>Link Coverage</div>
      <ProgressBar label="Upstream (target)" value={data.with_upstream} total={data.total_objects} pct={data.upstream_pct} />
      <ProgressBar label="Downstream (source)" value={data.with_downstream} total={data.total_objects} pct={data.downstream_pct} />
      <ProgressBar label="Any link" value={data.with_any_link} total={data.total_objects} pct={data.any_link_pct} />
    </div>
  );
}

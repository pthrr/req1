import { useEffect, useState } from "react";
import { api, type DashboardWidget as DashboardWidgetType, type WidgetDataEntry } from "./api/client";
import { theme } from "./theme";

interface Props {
  dashboardId: string;
  widget: DashboardWidgetType;
  onRefresh: () => void;
}

const BAR_COLORS = ["#1976d2", "#2e7d32", "#f57c00", "#d32f2f", "#7b1fa2", "#0097a7", "#c2185b", "#455a64"];

const SVG_WIDTH = 300;
const SVG_HEIGHT = 200;
const PADDING_TOP = 10;
const PADDING_BOTTOM = 40;
const PADDING_LEFT = 40;
const PADDING_RIGHT = 10;

const CHART_WIDTH = SVG_WIDTH - PADDING_LEFT - PADDING_RIGHT;
const CHART_HEIGHT = SVG_HEIGHT - PADDING_TOP - PADDING_BOTTOM;

export function DashboardWidget({ dashboardId, widget, onRefresh }: Props) {
  const [data, setData] = useState<WidgetDataEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    const fetchData = async () => {
      try {
        setLoading(true);
        const entries = await api.getWidgetData(dashboardId, widget.id);
        if (!cancelled) {
          setData(entries);
          setError(null);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : "Failed to load widget data");
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    };
    fetchData();
    return () => {
      cancelled = true;
    };
  }, [dashboardId, widget.id]);

  if (loading) {
    return (
      <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%", color: theme.colors.textMuted, fontSize: "0.8rem" }}>
        Loading...
      </div>
    );
  }

  if (error) {
    return (
      <div style={{ display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", height: "100%", gap: theme.spacing.xs }}>
        <span style={{ color: theme.colors.error, fontSize: "0.8rem" }}>{error}</span>
        <button
          onClick={onRefresh}
          style={{
            padding: "2px 8px",
            fontSize: "0.75rem",
            background: "none",
            color: theme.colors.primary,
            border: `1px solid ${theme.colors.primary}`,
            borderRadius: theme.borderRadius,
            cursor: "pointer",
          }}
        >
          Retry
        </button>
      </div>
    );
  }

  if (data.length === 0) {
    return (
      <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%", color: theme.colors.textMuted, fontSize: "0.8rem" }}>
        No data available
      </div>
    );
  }

  const maxValue = Math.max(...data.map((d) => d.value), 1);
  const barCount = data.length;
  const barGap = 4;
  const barWidth = Math.max(8, (CHART_WIDTH - barGap * (barCount - 1)) / barCount);

  // Y-axis tick count
  const tickCount = 4;
  const tickStep = maxValue / tickCount;

  return (
    <div style={{ display: "flex", flexDirection: "column", alignItems: "center", width: "100%", height: "100%" }}>
      <svg
        viewBox={`0 0 ${SVG_WIDTH} ${SVG_HEIGHT}`}
        style={{ width: "100%", maxHeight: "100%", overflow: "visible" }}
        preserveAspectRatio="xMidYMid meet"
      >
        {/* Y-axis gridlines and labels */}
        {Array.from({ length: tickCount + 1 }, (_, i) => {
          const val = Math.round(tickStep * i);
          const y = PADDING_TOP + CHART_HEIGHT - (val / maxValue) * CHART_HEIGHT;
          return (
            <g key={`tick-${i}`}>
              <line
                x1={PADDING_LEFT}
                y1={y}
                x2={PADDING_LEFT + CHART_WIDTH}
                y2={y}
                stroke={theme.colors.borderLight}
                strokeWidth={1}
              />
              <text
                x={PADDING_LEFT - 4}
                y={y + 3}
                textAnchor="end"
                fontSize="9"
                fill={theme.colors.textMuted}
              >
                {val}
              </text>
            </g>
          );
        })}

        {/* Bars */}
        {data.map((entry, i) => {
          const barHeight = (entry.value / maxValue) * CHART_HEIGHT;
          const x = PADDING_LEFT + i * (barWidth + barGap);
          const y = PADDING_TOP + CHART_HEIGHT - barHeight;
          const color = BAR_COLORS[i % BAR_COLORS.length];

          return (
            <g key={entry.label}>
              <rect
                x={x}
                y={y}
                width={barWidth}
                height={barHeight}
                fill={color}
                rx={2}
              >
                <title>{`${entry.label}: ${entry.value}`}</title>
              </rect>
              {/* Value label on top of bar */}
              {barHeight > 14 && (
                <text
                  x={x + barWidth / 2}
                  y={y + 10}
                  textAnchor="middle"
                  fontSize="8"
                  fill="#fff"
                  fontWeight="bold"
                >
                  {entry.value}
                </text>
              )}
              {/* X-axis label */}
              <text
                x={x + barWidth / 2}
                y={PADDING_TOP + CHART_HEIGHT + 12}
                textAnchor="middle"
                fontSize="8"
                fill={theme.colors.textSecondary}
                transform={`rotate(-30, ${x + barWidth / 2}, ${PADDING_TOP + CHART_HEIGHT + 12})`}
              >
                {entry.label.length > 12 ? entry.label.slice(0, 11) + "..." : entry.label}
              </text>
            </g>
          );
        })}

        {/* Axes */}
        <line
          x1={PADDING_LEFT}
          y1={PADDING_TOP}
          x2={PADDING_LEFT}
          y2={PADDING_TOP + CHART_HEIGHT}
          stroke={theme.colors.border}
          strokeWidth={1}
        />
        <line
          x1={PADDING_LEFT}
          y1={PADDING_TOP + CHART_HEIGHT}
          x2={PADDING_LEFT + CHART_WIDTH}
          y2={PADDING_TOP + CHART_HEIGHT}
          stroke={theme.colors.border}
          strokeWidth={1}
        />
      </svg>
    </div>
  );
}

import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { api, type Workspace } from "../api/client";
import { theme } from "../theme";

export function WorkspacePage() {
  const { workspaceId } = useParams<{ workspaceId: string }>();
  const [workspace, setWorkspace] = useState<Workspace | null>(null);

  useEffect(() => {
    if (workspaceId) {
      api.getWorkspace(workspaceId).then(setWorkspace).catch(() => {});
    }
  }, [workspaceId]);

  if (!workspace) return null;

  return (
    <div>
      <h1 style={{ marginTop: 0 }}>{workspace.name}</h1>
      <p style={{ color: theme.colors.textSecondary }}>
        {workspace.description ?? "No description."}
      </p>
      <p style={{ fontSize: "0.85rem", color: theme.colors.textMuted }}>
        Created: {new Date(workspace.created_at).toLocaleDateString()}
      </p>
    </div>
  );
}

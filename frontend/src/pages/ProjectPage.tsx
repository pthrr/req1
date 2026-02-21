import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { api, type Project } from "../api/client";
import { theme } from "../theme";

export function ProjectPage() {
  const { workspaceId, projectId } = useParams<{ workspaceId: string; projectId: string }>();
  const [project, setProject] = useState<Project | null>(null);

  useEffect(() => {
    if (workspaceId && projectId) {
      api.getProject(workspaceId, projectId).then(setProject).catch(() => {});
    }
  }, [workspaceId, projectId]);

  if (!project) return null;

  return (
    <div>
      <h1 style={{ marginTop: 0 }}>{project.name}</h1>
      <p style={{ color: theme.colors.textSecondary }}>
        {project.description ?? "No description."}
      </p>
      <p style={{ fontSize: "0.85rem", color: theme.colors.textMuted }}>
        Created: {new Date(project.created_at).toLocaleDateString()}
      </p>
    </div>
  );
}

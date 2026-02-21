import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { api, type Project } from "../api/client";
import { TraceabilityMatrix } from "../TraceabilityMatrix";

export function TraceabilityPage() {
  const { workspaceId, projectId } = useParams<{ workspaceId: string; projectId: string }>();
  const [project, setProject] = useState<Project | null>(null);

  useEffect(() => {
    if (workspaceId && projectId) {
      api.getProject(workspaceId, projectId).then(setProject).catch(() => {});
    }
  }, [workspaceId, projectId]);

  if (!project) return null;

  return <TraceabilityMatrix project={project} />;
}

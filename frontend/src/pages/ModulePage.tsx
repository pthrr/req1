import { useEffect, useState } from "react";
import { useParams } from "react-router";
import { api, type Module } from "../api/client";
import { ModuleDetail } from "../ModuleDetail";

export function ModulePage() {
  const { moduleId } = useParams<{ moduleId: string }>();
  const [module, setModule] = useState<Module | null>(null);

  useEffect(() => {
    if (moduleId) {
      api.getModule(moduleId).then(setModule).catch(() => {});
    }
  }, [moduleId]);

  if (!module) return null;

  return <ModuleDetail module={module} onModuleUpdated={setModule} />;
}

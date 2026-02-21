import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router";
import { App } from "./App";
import { WelcomePage } from "./pages/WelcomePage";
import { WorkspacePage } from "./pages/WorkspacePage";
import { ProjectPage } from "./pages/ProjectPage";
import { ModulePage } from "./pages/ModulePage";
import { TraceabilityPage } from "./pages/TraceabilityPage";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter>
      <Routes>
        <Route element={<App />}>
          <Route index element={<WelcomePage />} />
          <Route path="w/:workspaceId" element={<WorkspacePage />} />
          <Route path="w/:workspaceId/p/:projectId" element={<ProjectPage />} />
          <Route path="w/:workspaceId/p/:projectId/m/:moduleId" element={<ModulePage />} />
          <Route path="w/:workspaceId/p/:projectId/traceability" element={<TraceabilityPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  </StrictMode>,
);

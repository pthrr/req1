import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter, Routes, Route } from "react-router";
import { App } from "./App";
import { ThemeProvider } from "./ThemeContext";
import { AuthProvider } from "./AuthContext";
import { LoginPage } from "./LoginPage";
import { WelcomePage } from "./pages/WelcomePage";
import { WorkspacePage } from "./pages/WorkspacePage";
import { ProjectPage } from "./pages/ProjectPage";
import { ModulePage } from "./pages/ModulePage";
import { TraceabilityPage } from "./pages/TraceabilityPage";
import { DashboardListPage } from "./DashboardListPage";
import { DashboardPage } from "./DashboardPage";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ThemeProvider>
    <AuthProvider>
    <BrowserRouter>
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route element={<App />}>
          <Route index element={<WelcomePage />} />
          <Route path="w/:workspaceId" element={<WorkspacePage />} />
          <Route path="w/:workspaceId/p/:projectId" element={<ProjectPage />} />
          <Route path="w/:workspaceId/p/:projectId/m/:moduleId" element={<ModulePage />} />
          <Route path="w/:workspaceId/p/:projectId/traceability" element={<TraceabilityPage />} />
          <Route path="w/:workspaceId/dashboards" element={<DashboardListPage />} />
          <Route path="w/:workspaceId/dashboards/:dashboardId" element={<DashboardPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
    </AuthProvider>
    </ThemeProvider>
  </StrictMode>,
);

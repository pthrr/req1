import { theme } from "../theme";

export function WelcomePage() {
  return (
    <div style={{ display: "flex", alignItems: "center", justifyContent: "center", height: "100%", color: theme.colors.textMuted }}>
      <div style={{ textAlign: "center" }}>
        <h2 style={{ margin: 0, fontWeight: 400 }}>Welcome to req1</h2>
        <p>Select a workspace from the sidebar to get started.</p>
      </div>
    </div>
  );
}

import { useState } from "react";
import { useNavigate } from "react-router";
import { useAuth } from "./AuthContext";
import { useTheme } from "./ThemeContext";

export function LoginPage() {
  const { login, register } = useAuth();
  const navigate = useNavigate();
  const { theme } = useTheme();
  const [mode, setMode] = useState<"login" | "register">("login");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);
    try {
      if (mode === "login") {
        await login(email, password);
      } else {
        await register(email, password, displayName);
      }
      navigate("/");
    } catch (err) {
      setError(err instanceof Error ? err.message : "Authentication failed");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div
      style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        minHeight: "100vh",
        background: theme.colors.bg,
        fontFamily: theme.fontFamily,
      }}
    >
      <div
        style={{
          width: 380,
          padding: theme.spacing.lg,
          border: `1px solid ${theme.colors.border}`,
          borderRadius: theme.borderRadius,
          background: theme.colors.headerBg,
        }}
      >
        <h2 style={{ marginTop: 0, textAlign: "center", color: theme.colors.text }}>
          {mode === "login" ? "Sign In" : "Create Account"}
        </h2>

        {error && (
          <div style={{ color: theme.colors.error, marginBottom: theme.spacing.md, fontSize: "0.85rem", textAlign: "center" }}>
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: theme.spacing.md }}>
          <input
            type="email"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
            placeholder="Email"
            required
            style={{ padding: theme.spacing.sm, borderRadius: theme.borderRadius, border: `1px solid ${theme.colors.border}` }}
          />
          {mode === "register" && (
            <input
              type="text"
              value={displayName}
              onChange={(e) => setDisplayName(e.target.value)}
              placeholder="Display Name"
              required
              style={{ padding: theme.spacing.sm, borderRadius: theme.borderRadius, border: `1px solid ${theme.colors.border}` }}
            />
          )}
          <input
            type="password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            placeholder="Password"
            required
            style={{ padding: theme.spacing.sm, borderRadius: theme.borderRadius, border: `1px solid ${theme.colors.border}` }}
          />
          <button
            type="submit"
            disabled={loading}
            style={{
              padding: theme.spacing.sm,
              cursor: loading ? "not-allowed" : "pointer",
              background: theme.colors.primary,
              color: "#fff",
              border: "none",
              borderRadius: theme.borderRadius,
              fontWeight: 600,
            }}
          >
            {loading ? "..." : mode === "login" ? "Sign In" : "Register"}
          </button>
        </form>

        <div style={{ textAlign: "center", marginTop: theme.spacing.md, fontSize: "0.85rem" }}>
          {mode === "login" ? (
            <span style={{ color: theme.colors.textMuted }}>
              No account?{" "}
              <button
                onClick={() => { setMode("register"); setError(null); }}
                style={{ background: "none", border: "none", color: theme.colors.primary, cursor: "pointer", textDecoration: "underline" }}
              >
                Register
              </button>
            </span>
          ) : (
            <span style={{ color: theme.colors.textMuted }}>
              Have an account?{" "}
              <button
                onClick={() => { setMode("login"); setError(null); }}
                style={{ background: "none", border: "none", color: theme.colors.primary, cursor: "pointer", textDecoration: "underline" }}
              >
                Sign In
              </button>
            </span>
          )}
        </div>
      </div>
    </div>
  );
}

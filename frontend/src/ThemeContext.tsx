import { createContext, useCallback, useContext, useEffect, useState, type ReactNode } from "react";

type ThemeMode = "light" | "dark";

const lightColors = {
  bg: "#ffffff",
  sidebarBg: "#f8f9fa",
  sidebarBorder: "#e0e0e0",
  headerBg: "#ffffff",
  headerBorder: "#e0e0e0",
  primary: "#1976d2",
  danger: "#d32f2f",
  text: "#212121",
  textSecondary: "#616161",
  textMuted: "#9e9e9e",
  border: "#ccc",
  borderLight: "#eee",
  tabActive: "#1976d2",
  tabInactive: "#757575",
  error: "#d32f2f",
  success: "#2e7d32",
  suspect: "#e65100",
  bgHover: "#f0f0f0",
  bgCode: "#f5f5f5",
  overlayBg: "rgba(0, 0, 0, 0.5)",
};

const darkColors = {
  bg: "#1e1e1e",
  sidebarBg: "#252526",
  sidebarBorder: "#3c3c3c",
  headerBg: "#1e1e1e",
  headerBorder: "#3c3c3c",
  primary: "#4fc3f7",
  danger: "#ef5350",
  text: "#e0e0e0",
  textSecondary: "#b0b0b0",
  textMuted: "#757575",
  border: "#555",
  borderLight: "#333",
  tabActive: "#4fc3f7",
  tabInactive: "#888",
  error: "#ef5350",
  success: "#66bb6a",
  suspect: "#ff9800",
  bgHover: "#333",
  bgCode: "#2d2d2d",
  overlayBg: "rgba(0, 0, 0, 0.7)",
};

function buildTheme(mode: ThemeMode) {
  const colors = mode === "dark" ? darkColors : lightColors;
  return {
    mode,
    colors,
    spacing: {
      xs: "0.25rem",
      sm: "0.5rem",
      md: "1rem",
      lg: "1.5rem",
      xl: "2rem",
    },
    sidebar: {
      width: 280,
      indent: 20,
      itemPadding: "4px 8px",
    },
    header: {
      height: 48,
    },
    fontFamily: "system-ui, -apple-system, sans-serif",
    borderRadius: 4,
  } as const;
}

type Theme = ReturnType<typeof buildTheme>;

interface ThemeContextValue {
  theme: Theme;
  mode: ThemeMode;
  toggleMode: () => void;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [mode, setMode] = useState<ThemeMode>(() => {
    const stored = localStorage.getItem("req1-theme");
    return stored === "dark" ? "dark" : "light";
  });

  const toggleMode = useCallback(() => {
    setMode((prev) => {
      const next = prev === "light" ? "dark" : "light";
      localStorage.setItem("req1-theme", next);
      return next;
    });
  }, []);

  const themeValue = buildTheme(mode);

  // Apply body background for dark mode
  useEffect(() => {
    document.body.style.background = themeValue.colors.bg;
    document.body.style.color = themeValue.colors.text;
  }, [themeValue]);

  return (
    <ThemeContext.Provider value={{ theme: themeValue, mode, toggleMode }}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    // Fallback for components not wrapped in ThemeProvider
    return {
      theme: buildTheme("light"),
      mode: "light",
      toggleMode: () => {},
    };
  }
  return ctx;
}

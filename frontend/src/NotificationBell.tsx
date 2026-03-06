import { useCallback, useEffect, useState } from "react";
import { api, type Notification } from "./api/client";
import { theme } from "./theme";

export function NotificationBell() {
  const [unreadCount, setUnreadCount] = useState(0);
  const [open, setOpen] = useState(false);
  const [notifications, setNotifications] = useState<Notification[]>([]);

  const fetchCount = useCallback(async () => {
    try {
      const data = await api.getUnreadCount();
      setUnreadCount(data.count);
    } catch {
      // Non-critical
    }
  }, []);

  const fetchNotifications = useCallback(async () => {
    try {
      const data = await api.listNotifications({ limit: 20 });
      setNotifications(data.items);
    } catch {
      // Non-critical
    }
  }, []);

  useEffect(() => {
    fetchCount();
    const interval = setInterval(fetchCount, 30000);
    return () => clearInterval(interval);
  }, [fetchCount]);

  const handleOpen = () => {
    setOpen((prev) => !prev);
    if (!open) {
      fetchNotifications();
    }
  };

  const handleMarkRead = async (id: string) => {
    try {
      await api.markNotificationRead(id);
      setNotifications((prev) =>
        prev.map((n) => (n.id === id ? { ...n, read: true } : n)),
      );
      setUnreadCount((prev) => Math.max(0, prev - 1));
    } catch {
      // Non-critical
    }
  };

  const handleMarkAllRead = async () => {
    try {
      await api.markAllNotificationsRead();
      setNotifications((prev) => prev.map((n) => ({ ...n, read: true })));
      setUnreadCount(0);
    } catch {
      // Non-critical
    }
  };

  return (
    <div style={{ position: "relative" }}>
      <button
        onClick={handleOpen}
        style={{
          padding: "4px 10px",
          fontSize: "1rem",
          background: "none",
          border: `1px solid ${theme.colors.border}`,
          borderRadius: theme.borderRadius,
          cursor: "pointer",
          color: theme.colors.text,
          position: "relative",
        }}
        title="Notifications"
      >
        {"\\u{1F514}"}
        {unreadCount > 0 && (
          <span
            style={{
              position: "absolute",
              top: -4,
              right: -4,
              background: theme.colors.error,
              color: "#fff",
              borderRadius: "50%",
              width: 18,
              height: 18,
              fontSize: "0.7rem",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontWeight: 700,
            }}
          >
            {unreadCount > 99 ? "99+" : unreadCount}
          </span>
        )}
      </button>

      {open && (
        <div
          style={{
            position: "absolute",
            top: "100%",
            right: 0,
            width: 360,
            maxHeight: 400,
            overflowY: "auto",
            background: theme.colors.bg,
            border: `1px solid ${theme.colors.border}`,
            borderRadius: theme.borderRadius,
            boxShadow: "0 4px 16px rgba(0,0,0,0.15)",
            zIndex: 1100,
          }}
        >
          <div
            style={{
              display: "flex",
              justifyContent: "space-between",
              alignItems: "center",
              padding: `${theme.spacing.sm} ${theme.spacing.md}`,
              borderBottom: `1px solid ${theme.colors.borderLight}`,
            }}
          >
            <span style={{ fontWeight: 600, fontSize: "0.9rem" }}>
              Notifications
            </span>
            {unreadCount > 0 && (
              <button
                onClick={handleMarkAllRead}
                style={{
                  padding: "2px 8px",
                  fontSize: "0.8rem",
                  background: "none",
                  border: `1px solid ${theme.colors.border}`,
                  borderRadius: theme.borderRadius,
                  cursor: "pointer",
                  color: theme.colors.primary,
                }}
              >
                Mark all read
              </button>
            )}
          </div>
          {notifications.length === 0 ? (
            <div
              style={{
                padding: theme.spacing.lg,
                textAlign: "center",
                color: theme.colors.textMuted,
                fontSize: "0.85rem",
              }}
            >
              No notifications
            </div>
          ) : (
            notifications.map((n) => (
              <div
                key={n.id}
                onClick={() => {
                  if (!n.read) handleMarkRead(n.id);
                }}
                style={{
                  padding: `${theme.spacing.sm} ${theme.spacing.md}`,
                  borderBottom: `1px solid ${theme.colors.borderLight}`,
                  cursor: n.read ? "default" : "pointer",
                  background: n.read ? "transparent" : "#e3f2fd",
                  fontSize: "0.85rem",
                }}
              >
                <div style={{ fontWeight: n.read ? 400 : 600 }}>{n.title}</div>
                <div
                  style={{
                    color: theme.colors.textMuted,
                    fontSize: "0.8rem",
                    marginTop: 2,
                  }}
                >
                  {n.body.length > 80 ? n.body.slice(0, 80) + "..." : n.body}
                </div>
                <div
                  style={{
                    color: theme.colors.textMuted,
                    fontSize: "0.75rem",
                    marginTop: 2,
                  }}
                >
                  {new Date(n.created_at).toLocaleString()}
                </div>
              </div>
            ))
          )}
        </div>
      )}
    </div>
  );
}

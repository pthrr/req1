import { useCallback, useEffect, useRef, useState } from "react";
import type { AppUser } from "./api/client";
import { theme } from "./theme";

interface Props {
  value: string;
  onChange: (value: string) => void;
  users: AppUser[];
  placeholder?: string;
  rows?: number;
  style?: React.CSSProperties;
}

export function MentionTextarea({
  value,
  onChange,
  users,
  placeholder,
  rows = 3,
  style,
}: Props) {
  const [showDropdown, setShowDropdown] = useState(false);
  const [search, setSearch] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [mentionStart, setMentionStart] = useState(-1);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const filtered = users.filter(
    (u) =>
      u.active &&
      u.display_name.toLowerCase().includes(search.toLowerCase()),
  );

  const handleInput = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      const newValue = e.target.value;
      onChange(newValue);

      const cursorPos = e.target.selectionStart;
      // Find the last @ before cursor
      const textBeforeCursor = newValue.slice(0, cursorPos);
      const lastAt = textBeforeCursor.lastIndexOf("@");

      if (lastAt >= 0) {
        const textAfterAt = textBeforeCursor.slice(lastAt + 1);
        // Only show dropdown if no space in the mention text (or it's fresh)
        if (textAfterAt.length === 0 || !textAfterAt.includes("\n")) {
          setShowDropdown(true);
          setSearch(textAfterAt);
          setMentionStart(lastAt);
          setSelectedIndex(0);
          return;
        }
      }

      setShowDropdown(false);
    },
    [onChange],
  );

  const insertMention = useCallback(
    (user: AppUser) => {
      if (mentionStart < 0) return;
      const cursorPos = textareaRef.current?.selectionStart ?? value.length;
      const before = value.slice(0, mentionStart);
      const after = value.slice(cursorPos);
      const newValue = `${before}@${user.display_name} ${after}`;
      onChange(newValue);
      setShowDropdown(false);
      setSearch("");

      // Focus and set cursor position after insert
      setTimeout(() => {
        if (textareaRef.current) {
          const pos = mentionStart + user.display_name.length + 2; // @name + space
          textareaRef.current.focus();
          textareaRef.current.selectionStart = pos;
          textareaRef.current.selectionEnd = pos;
        }
      }, 0);
    },
    [mentionStart, value, onChange],
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (!showDropdown || filtered.length === 0) return;

      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((prev) => Math.min(prev + 1, filtered.length - 1));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((prev) => Math.max(prev - 1, 0));
      } else if (e.key === "Enter") {
        e.preventDefault();
        insertMention(filtered[selectedIndex]);
      } else if (e.key === "Escape") {
        setShowDropdown(false);
      }
    },
    [showDropdown, filtered, selectedIndex, insertMention],
  );

  // Close dropdown when clicking outside
  useEffect(() => {
    const handler = () => setShowDropdown(false);
    if (showDropdown) {
      document.addEventListener("click", handler);
      return () => document.removeEventListener("click", handler);
    }
  }, [showDropdown]);

  return (
    <div style={{ position: "relative", flex: 1 }}>
      <textarea
        ref={textareaRef}
        value={value}
        onChange={handleInput}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        rows={rows}
        style={{
          width: "100%",
          padding: theme.spacing.sm,
          resize: "vertical",
          boxSizing: "border-box",
          ...style,
        }}
      />
      {showDropdown && filtered.length > 0 && (
        <div
          style={{
            position: "absolute",
            bottom: "100%",
            left: 0,
            background: theme.colors.bg,
            border: `1px solid ${theme.colors.border}`,
            borderRadius: theme.borderRadius,
            boxShadow: "0 4px 12px rgba(0,0,0,0.15)",
            maxHeight: 200,
            overflowY: "auto",
            zIndex: 1100,
            minWidth: 200,
          }}
          onClick={(e) => e.stopPropagation()}
        >
          {filtered.map((user, i) => (
            <div
              key={user.id}
              onClick={() => insertMention(user)}
              style={{
                padding: "6px 12px",
                cursor: "pointer",
                background:
                  i === selectedIndex ? theme.colors.bgHover : "transparent",
                fontSize: "0.85rem",
              }}
              onMouseEnter={() => setSelectedIndex(i)}
            >
              <span style={{ fontWeight: 600 }}>{user.display_name}</span>
              <span
                style={{
                  marginLeft: 8,
                  color: theme.colors.textMuted,
                  fontSize: "0.8rem",
                }}
              >
                {user.email}
              </span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

/** Render text with @mentions highlighted */
export function renderMentionText(
  text: string,
  users: AppUser[],
): React.ReactNode {
  const parts: React.ReactNode[] = [];
  let lastIndex = 0;
  const regex = /@(\S+)/g;
  let match;

  while ((match = regex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push(text.slice(lastIndex, match.index));
    }
    const mentionName = match[1];
    const user = users.find(
      (u) => u.display_name.toLowerCase() === mentionName.toLowerCase(),
    );
    if (user) {
      parts.push(
        <span
          key={match.index}
          style={{
            background: "#e3f2fd",
            color: theme.colors.primary,
            borderRadius: 3,
            padding: "0 2px",
            fontWeight: 600,
          }}
        >
          @{user.display_name}
        </span>,
      );
    } else {
      parts.push(match[0]);
    }
    lastIndex = match.index + match[0].length;
  }

  if (lastIndex < text.length) {
    parts.push(text.slice(lastIndex));
  }

  return <>{parts}</>;
}

import { useCallback, useEffect, useRef } from "react";
import { useEditor, EditorContent } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Underline from "@tiptap/extension-underline";
import { Table, TableRow, TableCell, TableHeader } from "@tiptap/extension-table";
import Image from "@tiptap/extension-image";
import Placeholder from "@tiptap/extension-placeholder";
import { api } from "./api/client";
import { theme } from "./theme";

interface RichTextEditorProps {
  content: string;
  onChange: (html: string) => void;
  objectId: string;
  placeholder?: string;
  readOnly?: boolean;
  minHeight?: number;
}

export function RichTextEditor({
  content,
  onChange,
  objectId,
  placeholder = "Start writing...",
  readOnly = false,
  minHeight = 200,
}: RichTextEditorProps) {
  const fileInputRef = useRef<HTMLInputElement>(null);

  const uploadImage = useCallback(
    async (file: File) => {
      const attachment = await api.uploadAttachment(objectId, file);
      return api.downloadAttachmentUrl(objectId, attachment.id);
    },
    [objectId],
  );

  const editor = useEditor({
    extensions: [
      StarterKit,
      Underline,
      Table.configure({ resizable: true }),
      TableRow,
      TableCell,
      TableHeader,
      Image.configure({ inline: false, allowBase64: false }),
      Placeholder.configure({ placeholder }),
    ],
    content,
    editable: !readOnly,
    onUpdate: ({ editor: ed }) => {
      onChange(ed.getHTML());
    },
    editorProps: {
      handlePaste: (_view, event) => {
        const items = event.clipboardData?.items;
        if (!items) return false;
        for (const item of items) {
          if (item.type.startsWith("image/")) {
            event.preventDefault();
            const file = item.getAsFile();
            if (file) {
              uploadImage(file).then((url) => {
                editor?.chain().focus().setImage({ src: url }).run();
              });
            }
            return true;
          }
        }
        return false;
      },
      handleDrop: (_view, event) => {
        const files = event.dataTransfer?.files;
        if (!files || files.length === 0) return false;
        for (const file of files) {
          if (file.type.startsWith("image/")) {
            event.preventDefault();
            uploadImage(file).then((url) => {
              editor?.chain().focus().setImage({ src: url }).run();
            });
            return true;
          }
        }
        return false;
      },
    },
  });

  // Sync content from outside when it changes (e.g. initial load)
  const prevContentRef = useRef(content);
  useEffect(() => {
    if (editor && content !== prevContentRef.current) {
      prevContentRef.current = content;
      if (editor.getHTML() !== content) {
        editor.commands.setContent(content);
      }
    }
  }, [content, editor]);

  if (!editor) return null;

  const btn = (
    label: string,
    active: boolean,
    action: () => void,
    title?: string,
  ) => (
    <button
      type="button"
      onMouseDown={(e) => {
        e.preventDefault();
        action();
      }}
      title={title ?? label}
      style={{
        padding: "2px 6px",
        fontSize: "0.8rem",
        fontWeight: active ? 700 : 400,
        background: active ? theme.colors.primary : theme.colors.bgCode,
        color: active ? "#fff" : theme.colors.text,
        border: `1px solid ${theme.colors.border}`,
        borderRadius: 3,
        cursor: "pointer",
        minWidth: 28,
      }}
    >
      {label}
    </button>
  );

  return (
    <div>
      {/* Toolbar */}
      {!readOnly && (
        <div
          style={{
            display: "flex",
            gap: 2,
            flexWrap: "wrap",
            padding: "4px",
            borderBottom: `1px solid ${theme.colors.borderLight}`,
            marginBottom: 4,
          }}
        >
          {btn("B", editor.isActive("bold"), () => editor.chain().focus().toggleBold().run())}
          {btn("I", editor.isActive("italic"), () => editor.chain().focus().toggleItalic().run())}
          {btn("U", editor.isActive("underline"), () => editor.chain().focus().toggleUnderline().run())}
          {btn("S", editor.isActive("strike"), () => editor.chain().focus().toggleStrike().run())}
          <span style={{ borderLeft: `1px solid ${theme.colors.borderLight}`, margin: "0 2px" }} />
          {btn("H1", editor.isActive("heading", { level: 1 }), () => editor.chain().focus().toggleHeading({ level: 1 }).run())}
          {btn("H2", editor.isActive("heading", { level: 2 }), () => editor.chain().focus().toggleHeading({ level: 2 }).run())}
          {btn("H3", editor.isActive("heading", { level: 3 }), () => editor.chain().focus().toggleHeading({ level: 3 }).run())}
          <span style={{ borderLeft: `1px solid ${theme.colors.borderLight}`, margin: "0 2px" }} />
          {btn("\u2022", editor.isActive("bulletList"), () => editor.chain().focus().toggleBulletList().run(), "Bullet List")}
          {btn("1.", editor.isActive("orderedList"), () => editor.chain().focus().toggleOrderedList().run(), "Ordered List")}
          <span style={{ borderLeft: `1px solid ${theme.colors.borderLight}`, margin: "0 2px" }} />
          {btn("\u201C", editor.isActive("blockquote"), () => editor.chain().focus().toggleBlockquote().run(), "Blockquote")}
          {btn("</>", editor.isActive("codeBlock"), () => editor.chain().focus().toggleCodeBlock().run(), "Code Block")}
          <span style={{ borderLeft: `1px solid ${theme.colors.borderLight}`, margin: "0 2px" }} />
          {btn("Table", false, () => editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run())}
          {btn("Img", false, () => fileInputRef.current?.click(), "Insert Image")}
          <span style={{ borderLeft: `1px solid ${theme.colors.borderLight}`, margin: "0 2px" }} />
          {btn("\u21B6", false, () => editor.chain().focus().undo().run(), "Undo")}
          {btn("\u21B7", false, () => editor.chain().focus().redo().run(), "Redo")}
        </div>
      )}

      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        style={{ display: "none" }}
        onChange={async (e) => {
          const file = e.target.files?.[0];
          if (!file) return;
          const url = await uploadImage(file);
          editor.chain().focus().setImage({ src: url }).run();
          if (fileInputRef.current) fileInputRef.current.value = "";
        }}
      />

      <style>{`
        .ProseMirror {
          min-height: ${minHeight}px;
          padding: ${theme.spacing.sm};
          outline: none;
          font-size: 0.9rem;
        }
        .ProseMirror p.is-editor-empty:first-child::before {
          content: attr(data-placeholder);
          float: left;
          color: ${theme.colors.textMuted};
          pointer-events: none;
          height: 0;
        }
        .ProseMirror table {
          border-collapse: collapse;
          width: 100%;
          margin: 0.5em 0;
        }
        .ProseMirror th,
        .ProseMirror td {
          border: 1px solid ${theme.colors.border};
          padding: 6px 10px;
          text-align: left;
          min-width: 60px;
        }
        .ProseMirror th {
          background: ${theme.colors.bgCode};
          font-weight: 600;
        }
        .ProseMirror img {
          max-width: 100%;
          height: auto;
        }
        .ProseMirror blockquote {
          border-left: 3px solid ${theme.colors.border};
          padding-left: 12px;
          margin-left: 0;
          color: ${theme.colors.textSecondary};
        }
        .ProseMirror pre {
          background: ${theme.colors.bgCode};
          padding: 12px;
          border-radius: ${theme.borderRadius}px;
          overflow-x: auto;
        }
        .ProseMirror code {
          background: ${theme.colors.bgCode};
          padding: 2px 4px;
          border-radius: 3px;
          font-size: 0.85em;
        }
      `}</style>

      <div
        style={{
          border: `1px solid ${theme.colors.border}`,
          borderRadius: theme.borderRadius,
          overflow: "hidden",
        }}
      >
        <EditorContent editor={editor} />
      </div>
    </div>
  );
}

import { marked } from "marked";

const HTML_TAG_RE = /<(p|h[1-6]|ul|ol|table|img|div|blockquote|pre)\b/i;

export function isHtmlContent(body: string): boolean {
  return HTML_TAG_RE.test(body);
}

export function markdownToHtml(md: string): string {
  return marked.parse(md, { async: false }) as string;
}

export function prepareBodyForEditor(body: string | null): string {
  if (!body) return "";
  if (isHtmlContent(body)) return body;
  return markdownToHtml(body);
}

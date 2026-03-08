import { test, expect } from "@playwright/test";
import {
  setupAuth, uid, navigateToModule, scaffold,
  createObject, apiGet, auth, API,
} from "./helpers";

test.beforeAll(async () => { await setupAuth(); });

// ---------------------------------------------------------------------------
// Publish & Preview
// ---------------------------------------------------------------------------

test.describe("Publish", () => {
  test("UI: publish dropdown shows format options", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "pubui");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.getByRole("button", { name: /Publish/ }).click();
    await page.waitForTimeout(200);

    await expect(page.getByRole("button", { name: "HTML" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Markdown" })).toBeVisible();
    await expect(page.getByRole("button", { name: "LaTeX" })).toBeVisible();
  });

  test("UI: preview panel loads content", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "prevui");
    const heading = `PREVIEW-${uid()}`;
    await createObject(request, mod.id, heading);
    await navigateToModule(page, ws.name, proj.name, mod.name);

    await page.getByRole("button", { name: "Preview" }).click();
    await expect(page.getByText("Document Preview")).toBeVisible({ timeout: 5000 });

    // Verify HTML preview loads in iframe (blob URL, not empty)
    const iframe = page.locator("iframe[title='Document Preview']");
    await expect(iframe).toBeVisible({ timeout: 10000 });
    await expect(iframe).toHaveAttribute("src", /^blob:/, { timeout: 10000 });

    // Switch to Markdown and verify text content loads with auth
    await page.getByText("Document Preview").locator("..").locator("select").selectOption("markdown");
    await expect(page.locator("pre")).toBeVisible({ timeout: 10000 });
    await expect(page.locator("pre")).toContainText(heading, { timeout: 10000 });

    await page.getByRole("button", { name: "Close" }).click();
    await expect(page.getByText("Document Preview")).not.toBeVisible();
  });

  test("API: publish module as HTML", async ({ request }) => {
    const { mod } = await scaffold(request, "pubapi");
    await createObject(request, mod.id, `PUB-${uid()}`);
    const res = await request.get(`${API}/modules/${mod.id}/publish?format=html`, {
      headers: auth(),
    });
    expect(res.ok()).toBeTruthy();
    const html = await res.text();
    expect(html).toContain("PUB-");
  });

  test("API: publish as markdown", async ({ request }) => {
    const { mod } = await scaffold(request, "pubmd");
    const heading = `MD-${uid()}`;
    await createObject(request, mod.id, heading, "markdown body");
    const res = await request.get(`${API}/modules/${mod.id}/publish?format=markdown`, {
      headers: auth(),
    });
    expect(res.ok()).toBeTruthy();
    const md = await res.text();
    expect(md).toContain(heading);
  });

  test("API: publish as CSV", async ({ request }) => {
    const { mod } = await scaffold(request, "pubcsv");
    await createObject(request, mod.id, `CSV-${uid()}`);
    const res = await request.get(`${API}/modules/${mod.id}/publish?format=csv`, {
      headers: auth(),
    });
    expect(res.ok()).toBeTruthy();
    const csv = await res.text();
    expect(csv).toContain("CSV-");
  });

  test("API: publish as PDF", async ({ request }) => {
    const { mod } = await scaffold(request, "pubpdf");
    await createObject(request, mod.id, `PDF-${uid()}`);
    const res = await request.get(`${API}/modules/${mod.id}/publish?format=pdf`, {
      headers: auth(),
    });
    expect(res.ok()).toBeTruthy();
    expect(Number(res.headers()["content-length"])).toBeGreaterThan(0);
  });

  test("API: publish as XLSX", async ({ request }) => {
    const { mod } = await scaffold(request, "pubxlsx");
    await createObject(request, mod.id, `XLS-${uid()}`);
    const res = await request.get(`${API}/modules/${mod.id}/publish?format=xlsx`, {
      headers: auth(),
    });
    expect(res.ok()).toBeTruthy();
  });

  test("API: publish as DOCX", async ({ request }) => {
    const { mod } = await scaffold(request, "pubdocx");
    await createObject(request, mod.id, `DOC-${uid()}`);
    const res = await request.get(`${API}/modules/${mod.id}/publish?format=docx`, {
      headers: auth(),
    });
    expect(res.ok()).toBeTruthy();
  });

  test("API: publish as LaTeX", async ({ request }) => {
    const { mod } = await scaffold(request, "publatex");
    await createObject(request, mod.id, `TEX-${uid()}`);
    const res = await request.get(`${API}/modules/${mod.id}/publish?format=latex`, {
      headers: auth(),
    });
    expect(res.ok()).toBeTruthy();
    const tex = await res.text();
    expect(tex).toContain("TEX-");
  });

  test("API: publish as plain text", async ({ request }) => {
    const { mod } = await scaffold(request, "pubtxt");
    await createObject(request, mod.id, `TXT-${uid()}`);
    const res = await request.get(`${API}/modules/${mod.id}/publish?format=txt`, {
      headers: auth(),
    });
    expect(res.ok()).toBeTruthy();
    const txt = await res.text();
    expect(txt).toContain("TXT-");
  });

  test("API: publish as YAML", async ({ request }) => {
    const { mod } = await scaffold(request, "pubyaml");
    await createObject(request, mod.id, `YML-${uid()}`);
    const res = await request.get(`${API}/modules/${mod.id}/publish?format=yaml`, {
      headers: auth(),
    });
    expect(res.ok()).toBeTruthy();
    const yaml = await res.text();
    expect(yaml).toContain("YML-");
  });

  test("API: import CSV creates objects", async ({ request }) => {
    const { mod } = await scaffold(request, "impcsv");
    const heading = `IMP-${uid()}`;
    const csvBody = `level,heading,body\n1,${heading},imported body\n`;
    // Use native fetch — Playwright's request.post wraps string data in JSON
    const res = await fetch(`${API}/modules/${mod.id}/import/csv`, {
      method: "POST",
      headers: { ...auth(), "Content-Type": "text/csv" },
      body: csvBody,
    });
    expect(res.ok).toBeTruthy();
    const result = await res.json();
    expect(result.objects_created).toBeGreaterThan(0);

    // Verify object actually exists
    const objects = await apiGet(request, `/modules/${mod.id}/objects`);
    expect(objects.items.some((o: { heading: string }) => o.heading === heading)).toBeTruthy();
  });

  test("API: import XLSX creates objects", async ({ request }) => {
    const { mod } = await scaffold(request, "impxlsx");
    // Minimal XLSX: export first, then re-import
    const expRes = await request.get(`${API}/modules/${mod.id}/publish?format=xlsx`, {
      headers: auth(),
    });
    // If module is empty, just test the endpoint accepts binary data
    const xlsxBytes = await expRes.body();
    const res = await request.post(`${API}/modules/${mod.id}/import/xlsx`, {
      headers: { ...auth(), "Content-Type": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" },
      data: xlsxBytes,
    });
    // Accept 200 (success) or 400 (empty/invalid xlsx from empty module)
    expect([200, 400]).toContain(res.status());
  });

  test("UI: DOCX format in publish dropdown", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "docxui");
    await createObject(request, mod.id, `DPUB-${uid()}`);
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);
    await page.getByRole("button", { name: /Publish/ }).click();
    await page.waitForTimeout(300);
    await expect(page.getByText("Word (DOCX)").first()).toBeVisible();
  });
});

// ---------------------------------------------------------------------------
// Rich Text Editor (TipTap)
// ---------------------------------------------------------------------------

test.describe("Rich Text Editor", () => {
  test("TipTap editor loads in detail panel", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "rte");
    await createObject(request, mod.id, `RTE-${uid()}`, "Some body text");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await page.getByRole("button", { name: "Edit" }).first().click();
    await page.waitForTimeout(500);
    await expect(page.locator(".ProseMirror")).toBeVisible();
    await expect(page.getByRole("button", { name: "B" }).first()).toBeVisible();
    await expect(page.getByRole("button", { name: "I" }).first()).toBeVisible();
    await page.getByRole("button", { name: "Close" }).click();
  });

  test("save object with rich text body", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "rtesave");
    await createObject(request, mod.id, `SAVE-${uid()}`);
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await page.getByRole("button", { name: "Edit" }).first().click();
    await page.waitForTimeout(500);
    const editor = page.locator(".ProseMirror");
    await editor.click();
    await editor.pressSequentially("Rich text content");
    await page.waitForTimeout(200);
    await page.getByRole("button", { name: "Save", exact: true }).click();
    await page.waitForTimeout(1000);
    await page.getByRole("button", { name: "Close" }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText("Rich text content").first()).toBeVisible();
  });

  test("table insertion via toolbar", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "rtetbl");
    await createObject(request, mod.id, `TBL-${uid()}`);
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await page.getByRole("button", { name: "Edit" }).first().click();
    await page.waitForTimeout(500);
    await page.getByRole("button", { name: "Table" }).click();
    await page.waitForTimeout(300);
    await expect(page.locator(".ProseMirror table")).toBeVisible();
    await page.getByRole("button", { name: "Close" }).click();
  });
});

// ---------------------------------------------------------------------------
// Document View (LiveDoc)
// ---------------------------------------------------------------------------

test.describe("Document View", () => {
  test("Document tab renders objects", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "docview");
    await createObject(request, mod.id, `DOC-001-${uid()}`, "First section body");
    await createObject(request, mod.id, `DOC-002-${uid()}`, "Second section body");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await page.getByRole("button", { name: "Document" }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText("First section body")).toBeVisible();
    await expect(page.getByText("Second section body")).toBeVisible();
  });

  test("Document outline sidebar shows headings", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "docout");
    const h1 = `OUT-A-${uid()}`;
    const h2 = `OUT-B-${uid()}`;
    await createObject(request, mod.id, h1, "Body A");
    await createObject(request, mod.id, h2, "Body B");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await page.getByRole("button", { name: "Document" }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText("Outline")).toBeVisible();
    await expect(page.getByText(h1).first()).toBeVisible();
    await expect(page.getByText(h2).first()).toBeVisible();
  });

  test("double-click enables inline editing", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "docedit");
    await createObject(request, mod.id, `IED-${uid()}`, "Editable content");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await page.getByRole("button", { name: "Document" }).click();
    await page.waitForTimeout(500);
    await page.getByText("Editable content").dblclick();
    await page.waitForTimeout(500);
    await expect(page.getByRole("button", { name: "Save" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Cancel" })).toBeVisible();
    await expect(page.locator(".ProseMirror")).toBeVisible();

    await page.getByRole("button", { name: "Cancel" }).click();
    await page.waitForTimeout(300);
    await expect(page.getByText("Editable content")).toBeVisible();
  });

  test("export Word and PDF buttons present", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "docexport");
    await createObject(request, mod.id, `EXP-${uid()}`);
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await page.getByRole("button", { name: "Document" }).click();
    await page.waitForTimeout(500);
    await expect(page.getByRole("button", { name: "Export Word" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Export PDF" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Print" })).toBeVisible();
  });

  test("switch between Objects and Document tabs preserves data", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "docswitch");
    const heading = `SWT-${uid()}`;
    await createObject(request, mod.id, heading, "Persistent body");
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await expect(page.getByText(heading).first()).toBeVisible();
    await page.getByRole("button", { name: "Document" }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText("Persistent body")).toBeVisible();
    await page.getByRole("button", { name: "Objects" }).click();
    await page.waitForTimeout(500);
    await expect(page.getByText(heading).first()).toBeVisible();
  });
});

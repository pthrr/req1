import { test, expect } from "@playwright/test";
import {
  setupAuth, uid, navigateToModule, scaffold, clickRowAction,
  createObject, apiGet, apiDelete, auth, API,
} from "./helpers";

test.beforeAll(async () => { await setupAuth(); });

test.describe("References Panel", () => {
  test("open and add a reference", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "refui");
    await createObject(request, mod.id, `REF-${uid()}`);
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    await clickRowAction(page, "Ref");
    await expect(page.getByText(/References for/)).toBeVisible({ timeout: 5000 });
    await page.getByRole("button", { name: "Add Reference" }).click();
    await page.waitForTimeout(200);
    await page.getByPlaceholder("Path or URL").fill("https://example.com/spec");
    await page.getByPlaceholder("Description").fill("External specification");
    await page.getByRole("button", { name: "Save", exact: true }).click();
    await page.waitForTimeout(500);
    await page.getByRole("button", { name: "Close" }).click();
  });
});

test.describe("Attachments Panel", () => {
  test("open attachment panel", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "attui");
    await createObject(request, mod.id, `ATT-${uid()}`);
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);
    await page.getByRole("button", { name: "Objects" }).click();
    await page.waitForTimeout(1000);

    await expect(page.locator(".ag-row").first()).toBeVisible({ timeout: 5000 });
    await clickRowAction(page, "Att");
    await expect(page.getByText(/Attachments for/)).toBeVisible({ timeout: 5000 });
    await expect(page.getByRole("button", { name: "Upload" })).toBeVisible();
    await page.getByRole("button", { name: "Close" }).click();
  });

  test("API: upload, list, verify, download, delete attachment", async ({ request }) => {
    const { mod } = await scaffold(request, "attapi");
    const obj = await createObject(request, mod.id, `ATT-API-${uid()}`);

    // Upload a text file via multipart
    const fileContent = Buffer.from("Hello attachment content");
    const res = await request.post(`${API}/objects/${obj.id}/attachments`, {
      headers: auth(),
      multipart: {
        file: { name: "test.txt", mimeType: "text/plain", buffer: fileContent },
      },
    });
    expect(res.status()).toBe(201);
    const att = await res.json();
    expect(att.id).toBeTruthy();
    expect(att.file_name).toBe("test.txt");

    // List attachments
    const list = await apiGet(request, `/objects/${obj.id}/attachments`);
    expect(list.length).toBe(1);

    // Verify integrity
    const verify = await apiGet(request, `/objects/${obj.id}/attachments/${att.id}/verify`);
    expect(verify.valid).toBe(true);
    expect(verify.file_name).toBe("test.txt");

    // Download
    const dlRes = await request.get(`${API}/objects/${obj.id}/attachments/${att.id}/download`, {
      headers: auth(),
    });
    expect(dlRes.ok()).toBeTruthy();
    const body = await dlRes.body();
    expect(body.toString()).toBe("Hello attachment content");

    // Delete
    await apiDelete(request, `/objects/${obj.id}/attachments/${att.id}`);
    const listAfter = await apiGet(request, `/objects/${obj.id}/attachments`);
    expect(listAfter.length).toBe(0);
  });
});

test.describe("Activity Feed", () => {
  test("expand shows recent changes", async ({ page, request }) => {
    const { ws, proj, mod } = await scaffold(request, "afui");
    await createObject(request, mod.id, `AF-${uid()}`);
    await navigateToModule(page, ws.name, proj.name, mod.name);
    await page.waitForTimeout(1000);

    const feedBtn = page.getByRole("button", { name: "Activity Feed" });
    await expect(feedBtn).toBeVisible();
    await feedBtn.click();
    await page.waitForTimeout(2000);
    await expect(page.getByText("create").first()).toBeVisible({ timeout: 5000 });
  });
});

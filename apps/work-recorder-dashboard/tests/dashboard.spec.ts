import { expect, test, type Page } from "@playwright/test";
import path from "node:path";

const artifactDir = path.resolve(process.cwd(), "../../target/ctx-artifacts/dashboard-react");

test("desktop light populated dashboard", async ({ page }, testInfo) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Work Records" })).toBeVisible();
  await expect(page.getByText("Finish dashboard React export")).toBeVisible();
  await expect(page.getByText("Share-safe export")).toBeVisible();
  await expect(page.getByText("1 failing command")).toBeVisible();
  await assertNonBlank(page);
  await screenshot(page, testInfo.project.name, "desktop-light-overview");
});

test("desktop dark session transcript and commands", async ({ page }, testInfo) => {
  await page.goto("/");
  await page.getByTitle("Use dark theme").click();
  await page.getByRole("tab", { name: "Session" }).click();
  await expect(page.getByRole("heading", { name: "Transcript, Messages, and Tool Calls" })).toBeVisible();
  await expect(page.getByText("exec_command npm run build")).toBeVisible();
  await expect(page.getByText("cargo test -p work-record-report")).toBeVisible();
  await assertNonBlank(page);
  await screenshot(page, testInfo.project.name, "desktop-dark-session");
});

test("mobile evidence failure state", async ({ page }, testInfo) => {
  await page.goto("/");
  await page.getByRole("tab", { name: "PR/Evidence" }).click();
  await expect(page.getByRole("heading", { name: "Evidence Previews" })).toBeVisible();
  await expect(page.getByText("buildkite-agent pipeline upload")).toBeVisible();
  await expect(page.getByText("missing BUILDKITE_AGENT_TOKEN")).toBeVisible();
  await assertNonBlank(page);
  await screenshot(page, testInfo.project.name, "mobile-evidence-failure");
});

test("mobile status and search", async ({ page }, testInfo) => {
  await page.goto("/");
  await page.getByRole("tab", { name: "Search" }).click();
  await page.getByPlaceholder("Search records, commands, transcript previews, artifacts").fill("provider");
  await expect(page.getByText("Import provider fixture sessions")).toBeVisible();
  await page.getByRole("tab", { name: "Status" }).click();
  await expect(page.getByRole("heading", { name: "Settings / Status" })).toBeVisible();
  await expect(page.getByText("Work Recorder dashboard export v1")).toBeVisible();
  await assertNonBlank(page);
  await screenshot(page, testInfo.project.name, "mobile-status-search");
});

async function screenshot(page: Page, project: string, name: string) {
  await page.screenshot({
    path: path.join(artifactDir, `${project}-${name}.png`),
    fullPage: true
  });
}

async function assertNonBlank(page: Page) {
  const sample = await page.evaluate(() => {
    const rect = document.body.getBoundingClientRect();
    const textLength = document.body.innerText.trim().length;
    const elements = document.querySelectorAll("section, article, table, [role='tab']").length;
    return { width: rect.width, height: rect.height, textLength, elements };
  });
  expect(sample.width).toBeGreaterThan(300);
  expect(sample.height).toBeGreaterThan(500);
  expect(sample.textLength).toBeGreaterThan(400);
  expect(sample.elements).toBeGreaterThan(6);
}

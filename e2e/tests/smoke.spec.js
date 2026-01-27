import { test, expect } from "@playwright/test";

const apiBaseUrl =
  process.env.E2E_API_BASE_URL ?? "http://127.0.0.1:8000/api";

test("frontend loads and backend is healthy", async ({ page, request }) => {
  const response = await request.get(`${apiBaseUrl}/health`);
  expect(response.ok()).toBeTruthy();
  await expect(await response.text()).toBe("OK");

  await page.goto("/");
  await expect(page).toHaveTitle(/Reader/);
});

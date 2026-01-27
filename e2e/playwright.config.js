import { defineConfig } from "@playwright/test";
import dotenv from "dotenv";
import path from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
dotenv.config({ path: path.join(__dirname, ".env.e2e") });

const baseURL = process.env.E2E_BASE_URL ?? "http://127.0.0.1:3000";
const apiBaseURL = process.env.E2E_API_BASE_URL ?? "http://127.0.0.1:8000/api";

export default defineConfig({
  testDir: "./tests",
  timeout: 60_000,
  expect: {
    timeout: 10_000,
  },
  use: {
    baseURL,
    trace: "on-first-retry",
  },
  webServer: [
    {
      command: "node ./scripts/backend-with-db.js",
      url: `${apiBaseURL}/health`,
      timeout: 120_000,
      reuseExistingServer: false,
    },
    {
      command: "pnpm --dir ../frontend dev --host 127.0.0.1 --port 3000",
      url: baseURL,
      timeout: 120_000,
      reuseExistingServer: true,
      env: {
        VITE_API_BASE_URL: apiBaseURL,
      },
    },
  ],
});

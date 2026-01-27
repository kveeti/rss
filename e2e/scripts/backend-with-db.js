import { PostgreSqlContainer } from "@testcontainers/postgresql";
import { spawn } from "node:child_process";
import dotenv from "dotenv";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";
import { setTimeout as delay } from "node:timers/promises";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
dotenv.config({ path: path.join(__dirname, "..", ".env.e2e") });

const backendHost = process.env.HOST ?? "127.0.0.1:8000";
const frontBaseUrl = process.env.FRONT_BASE_URL ?? "http://127.0.0.1:3000";
const apiBaseUrl = process.env.E2E_API_BASE_URL ?? "http://127.0.0.1:8000/api";
const postgresImage = process.env.E2E_POSTGRES_IMAGE ?? "postgres:18-alpine";

const healthUrl = `${apiBaseUrl.replace(/\/$/, "")}/health`;

const container = await new PostgreSqlContainer(postgresImage)
  .withDatabase("db")
  .withUsername("pg")
  .withPassword("pg")
  .start();

const databaseUrl = container.getConnectionUri();

const backendProcess = spawn("cargo", ["run"], {
  cwd: path.resolve(__dirname, "..", "..", "backend"),
  env: {
    ...process.env,
    DATABASE_URL: databaseUrl,
    HOST: backendHost,
    FRONT_BASE_URL: frontBaseUrl,
  },
  stdio: "inherit",
});

backendProcess.on("exit", (code) => {
  if (code !== 0) {
    console.error(`backend exited with code ${code}`);
    process.exitCode = code ?? 1;
  }
});

const waitForHealth = async (timeoutMs) => {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const response = await fetch(healthUrl);
      if (response.ok) {
        return;
      }
    } catch {
      // ignore and retry
    }
    await delay(500);
  }
  throw new Error(`backend did not become healthy at ${healthUrl}`);
};

const shutdown = async (signal) => {
  console.log(`shutting down (${signal})...`);
  if (backendProcess && !backendProcess.killed) {
    backendProcess.kill("SIGINT");
    await Promise.race([
      new Promise((resolve) => backendProcess.once("exit", resolve)),
      delay(5_000),
    ]);
  }
  await container.stop();
  process.exit(0);
};

process.on("SIGINT", () => shutdown("SIGINT"));
process.on("SIGTERM", () => shutdown("SIGTERM"));

await waitForHealth(60_000);
console.log("backend ready");

await new Promise(() => {});

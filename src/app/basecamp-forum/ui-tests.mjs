#!/usr/bin/env node

import { fileURLToPath } from "node:url";
import { dirname, resolve } from "node:path";

const __dirname = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(__dirname, "../..");
const qtMcpRoot = process.env.LOGOS_QT_MCP || resolve(repoRoot, "result-mcp");
const { test, run } = await import(resolve(qtMcpRoot, "test-framework/framework.mjs"));

async function openLp0016(app) {
  const candidates = [
    "lp0016_anon_forum",
    "LP-0016 Anonymous Forum Demo",
    "LP-0016",
  ];
  for (const label of candidates) {
    try {
      await app.click(label);
      await app.waitFor(
        async () => {
          await app.expectTexts(["LP-0016", "Forum"]);
        },
        { timeout: 5000, interval: 250, description: `LP-0016 app after clicking ${label}` },
      );
      return;
    } catch (_err) {
      // Try the next label; Basecamp has changed sidebar labels across releases.
    }
  }
  await app.expectTexts(["LP-0016", "Forum"]);
}

test("lp0016_basecamp_forum: click through the full moderation flow", async (app) => {
  await openLp0016(app);

  const screens = [
    "Register",
    "Post",
    "Moderate",
    "Vote",
    "Certificate",
    "History",
    "Slash",
    "Rejected",
  ];

  for (const screen of screens) {
    await app.click("Next");
    await app.waitFor(
      async () => {
        await app.expectTexts([screen]);
      },
      { timeout: 5000, interval: 250, description: `${screen} screen` },
    );
  }

  await app.expectTexts(["member revoked", "receipt cannot prove non-membership"]);
});

test("lp0016_basecamp_forum: direct sidebar navigation reaches slash screen", async (app) => {
  await openLp0016(app);
  await app.click("8. Slash");
  await app.expectTexts(["slash", "commitment reconstructed", "Submit slash"]);
});

run();

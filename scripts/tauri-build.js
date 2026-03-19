#!/usr/bin/env node

const { spawnSync } = require("node:child_process");

const env = { ...process.env };

if (process.platform === "linux") {
  env.NO_STRIP = env.NO_STRIP || "1";
}

const result = spawnSync("npx", ["tauri", "build"], {
  env,
  stdio: "inherit",
  shell: true,
});

if (typeof result.status === "number") {
  process.exit(result.status);
}

process.exit(1);

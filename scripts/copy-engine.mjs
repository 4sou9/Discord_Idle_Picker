// Builds the dummy helper binary and stages it into src-tauri/engine/ so
// tauri.conf.json's `bundle.resources` can pick it up.
// Run manually with `node scripts/copy-engine.mjs` or automatically via
// `predev` / tauri.conf.json's `beforeBuildCommand`.
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const engineDir = join(root, "src-tauri", "engine");

console.log("[copy-engine] building dummy (release)...");
execFileSync("cargo", ["build", "--release", "-p", "dummy"], {
  cwd: root,
  stdio: "inherit",
});

mkdirSync(engineDir, { recursive: true });

const dummyExe = join(root, "target", "release", "dummy.exe");
if (!existsSync(dummyExe)) {
  throw new Error(`dummy.exe not found at ${dummyExe}`);
}

copyFileSync(dummyExe, join(engineDir, "dummy.exe"));
console.log(`[copy-engine] staged engine files into ${engineDir}`);

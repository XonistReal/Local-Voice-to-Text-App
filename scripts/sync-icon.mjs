import { copyFileSync, existsSync } from "node:fs";
import { execSync } from "node:child_process";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const sourceIco = resolve(root, "assets", "source-icon.ico");
const iconsDir = resolve(root, "src-tauri", "icons");

if (!existsSync(sourceIco)) {
  console.log("assets/source-icon.ico not found — skipping icon sync");
  process.exit(0);
}

execSync(`npx tauri icon "${sourceIco}"`, { cwd: root, stdio: "inherit" });

// Use tauri-generated multi-resolution icon.ico for Windows exe embedding.
// Do NOT overwrite with the raw source file — it lacks required resolutions.

const titleBarIcon = resolve(iconsDir, "icon.png");
const fallbackIcon = resolve(iconsDir, "128x128@2x.png");
const publicIcon = resolve(root, "public", "app-icon.png");

if (existsSync(titleBarIcon)) {
  copyFileSync(titleBarIcon, publicIcon);
} else if (existsSync(fallbackIcon)) {
  copyFileSync(fallbackIcon, publicIcon);
}

console.log("Icons synced from assets/source-icon.ico");

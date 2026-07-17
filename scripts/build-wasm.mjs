// Builds the usfm_onion_wasm package for a given wasm-bindgen target.
//
// Usage:
//   node scripts/build-wasm.mjs bundler          # ships: wasm-release-fast (O3)
//   node scripts/build-wasm.mjs web
//   node scripts/build-wasm.mjs bundler --oz      # reference point: wasm-release (Oz)
//   node scripts/build-wasm.mjs web --oz
//   node scripts/build-wasm.mjs bundler --dev
//   node scripts/build-wasm.mjs web --dev
//
// Release builds (`--oz` or the O3 default) use a dedicated Cargo profile
// plus an explicit `wasm-opt` pass — see the root Cargo.toml for why this
// isn't just `wasm-pack build --release` (that maps to the *native*-tuned
// `release` profile now, and `wasm-pack build --profile <name>` doesn't
// read the wasm-opt flags from Cargo.toml metadata the way `--release`
// does). Requires `wasm-opt` on PATH (`brew install binaryen` / `apt
// install binaryen`). Dev builds skip all of that for fast iteration +
// debug info.

import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";

const ROOT = fileURLToPath(new URL("..", import.meta.url));
const CRATE_DIR = "crates/usfm_onion_wasm";

const [, , target, ...rest] = process.argv;
const dev = rest.includes("--dev");
const oz = rest.includes("--oz");

if (target !== "bundler" && target !== "web") {
  console.error("usage: node scripts/build-wasm.mjs <bundler|web> [--dev|--oz]");
  process.exit(1);
}

const outDir = target === "bundler" ? "pkg-bundler" : "pkg-web";

function run(command, args) {
  console.log(`$ ${command} ${args.join(" ")}`);
  execFileSync(command, args, { cwd: ROOT, stdio: "inherit" });
}

if (dev) {
  run("wasm-pack", [
    "build",
    CRATE_DIR,
    "--target",
    target,
    "--dev",
    "--out-dir",
    `../../${outDir}`,
    "--out-name",
    "usfm_onion_web",
  ]);
} else {
  const cargoProfile = oz ? "wasm-release" : "wasm-release-fast";
  const wasmOptLevel = oz ? "-Oz" : "-O3";
  run("wasm-pack", [
    "build",
    CRATE_DIR,
    "--target",
    target,
    "--profile",
    cargoProfile,
    "--out-dir",
    `../../${outDir}`,
    "--out-name",
    "usfm_onion_web",
    "--no-opt",
  ]);
  run("wasm-opt", [
    wasmOptLevel,
    "--enable-bulk-memory",
    path.join(outDir, "usfm_onion_web_bg.wasm"),
    "-o",
    path.join(outDir, "usfm_onion_web_bg.wasm"),
  ]);
}

run("node", [path.join("scripts", "restore-wasm-package-layout.mjs")]);

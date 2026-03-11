import { writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("..", import.meta.url));
const PACKAGE_DIRS = ["pkg-web", "pkg-bundler"];
const GITIGNORE_CONTENT = `# Intentionally checked in: downstream consumes these generated artifacts.
# wasm-pack rewrites this file during build, so restore it after each package build.

node_modules
.DS_Store

!.gitignore
!README.md
!package.json
!*.d.ts
!*.js
!*.wasm
`;

for (const dir of PACKAGE_DIRS) {
  writeFileSync(join(ROOT, dir, ".gitignore"), GITIGNORE_CONTENT);
}

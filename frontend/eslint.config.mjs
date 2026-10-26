import { createRequire } from "module";
import path from "path";
import { fileURLToPath } from "url";

// `__filename` is not dead: `__dirname` below is derived from it and is part of the
// ESM/CJS interop this config relies on. Do not delete it.
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// `eslint-config-next` is a CommonJS package, but this file is an ESM flat config, so a
// plain `import` of it is not portable. `createRequire(import.meta.url)` is the standard
// Node bridge from ESM to CommonJS; it has existed since Node 12.2.0, while the project's
// effective minimum is Node 18.18.0 (the floor required by ESLint 9.17.0 / ESLint v9).
const require = createRequire(import.meta.url);

const nextCoreWebVitals = require("eslint-config-next/core-web-vitals");

// The `Array.isArray` guard below is defensive: `eslint-config-next` is CommonJS and its
// `next/core-web-vitals` entrypoint currently exports a flat-config **array** of 4 config
// objects (verified with the pinned eslint-config-next 16.0.0; older 14.2.x/15.x releases
// also export an array). No release exports a stable, non-array shape yet. TODO: once a
// future `eslint-config-next` major ships a stable, non-array flat-config export, simplify
// this to the direct value and remove the guard — re-check on every major upgrade, and do
// not remove the guard while the export may still be an array.
export default [
  {
    ignores: [".next/**", "node_modules/**"],
  },
  ...(Array.isArray(nextCoreWebVitals) ? nextCoreWebVitals : [nextCoreWebVitals]).map((config) => ({
    ...config,
    // Include .mjs/.cjs so config and tooling files (including this file) are linted
    // under the same rule set instead of being silently skipped.
    files: ["**/*.{js,jsx,ts,tsx,mjs,cjs}"],
  })),
];

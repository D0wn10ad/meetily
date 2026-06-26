import { createRequire } from "module";
import path from "path";
import { fileURLToPath } from "url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const require = createRequire(import.meta.url);

const nextCoreWebVitals = require("eslint-config-next/core-web-vitals");

export default [
  {
    ignores: [".next/**", "node_modules/**"],
  },
  ...(Array.isArray(nextCoreWebVitals) ? nextCoreWebVitals : [nextCoreWebVitals]).map((config) => ({
    ...config,
    files: ["**/*.{js,jsx,ts,tsx}"],
  })),
];

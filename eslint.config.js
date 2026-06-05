// ESLint v9 flat config — TypeScript + React (Vite) frontend.
//
// Scope: `src/**` plus this repo's root config files. The Rust backend
// (`src-tauri/`), generated `dist/`, character asset packs, and the Python
// hook scripts are linted by their own toolchains and excluded here. Uses
// the non-type-checked `recommended` preset so the lint gate stays fast in
// CI — type errors are already enforced by `tsc --noEmit` (the `typecheck`
// script), so we don't pay the project-service cost twice.

import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";

export default tseslint.config(
  {
    // Not authored by us / linted elsewhere.
    ignores: [
      "dist/**",
      "node_modules/**",
      "src-tauri/**",
      "characters/**",
      "public/**",
      "scripts/**",
      "schemas/**",
      "hooks/**",
      "plans/**",
      "docs/**",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: 2022,
      globals: {
        ...globals.browser,
      },
    },
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      "react-refresh/only-export-components": [
        "warn",
        { allowConstantExport: true },
      ],
      // Underscore-prefixed args/vars are intentional "unused" markers —
      // mirrors the tsconfig `noUnusedParameters` convention already used
      // across the renderer (e.g. `_crossfadeMs`, `_voice`, `_app`).
      "@typescript-eslint/no-unused-vars": [
        "error",
        {
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
        },
      ],
    },
  },
  {
    // Build tooling runs in Node, not the browser.
    files: ["*.{ts,js}", "vite.config.ts"],
    languageOptions: {
      globals: {
        ...globals.node,
      },
    },
  },
);

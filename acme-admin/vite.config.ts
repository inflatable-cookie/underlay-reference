/// <reference types="node" />

import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vitest/config";
import type { PluginOption } from "vite";
import { fileURLToPath } from "node:url";
import devtoolsJson from "vite-plugin-devtools-json";

const apiClientSrc = fileURLToPath(
  new URL("../acme-client/src", import.meta.url),
);

export default defineConfig({
  plugins: [devtoolsJson(), sveltekit() as PluginOption],
  test: {
    include: ["tests/**/*.{test,spec}.ts"],
    environment: "jsdom",
    globals: true,
    setupFiles: ["tests/setup.ts"],
  },
  resolve: {
    alias: {
      "@api-client": apiClientSrc,
      "acme-client": apiClientSrc,
    },
    dedupe: ["@decodelabs/underlay"],
  },
  optimizeDeps: {
    // Underlay is a local `file:` dependency and changes frequently.
    exclude: [
      "@decodelabs/underlay",
      "@decodelabs/underlay/runtime",
      "@decodelabs/underlay/styles",
      "@decodelabs/underlay/client",
    ],
  },
  ssr: {
    noExternal: [
      "bits-ui",
      "runed",
      "svelte-toolbelt",
      "lucide-svelte",
      "easymde",
    ],
  },
  server: {
    port: 41002,
    strictPort: true,
    allowedHosts: [
      "acme.test",
      "admin.acme.test",
      "api.acme.test",
    ],
    watch: {
      // Watch changes in symlinked local dependencies
      ignored: [
        "!**/node_modules/@decodelabs/underlay/**",
        "!**/node_modules/@compli-me/ui/**",
        "!**/node_modules/acme-client/**",
      ],
    },
  },
  preview: {
    port: 41002,
    strictPort: true,
  },
});

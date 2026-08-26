/// <reference types="node" />

import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vitest/config";
import type { PluginOption } from "vite";
import { fileURLToPath } from "node:url";

const apiClientSrc = fileURLToPath(
  new URL("../../packages/acme-client/src", import.meta.url),
);

export default defineConfig({
  plugins: [
    {
      name: "decode-encoded-fs-paths",
      enforce: "pre",
      configureServer(server) {
        server.middlewares.use((req, _res, next) => {
          const url = req.url;
          if (!url || !url.startsWith("/@fs/")) {
            next();
            return;
          }

          const [pathname, search] = url.split("?", 2);
          try {
            const decodedPathname = decodeURIComponent(pathname);
            if (decodedPathname !== pathname) {
              req.url = decodedPathname + (search ? `?${search}` : "");
            }
          } catch {
            // If decoding fails, fall back to original URL.
          }

          next();
        });
      },
    },
    sveltekit() as PluginOption,
  ],
  build: {
    chunkSizeWarningLimit: 1200,
  },
  resolve: {
    alias: {
      "@api-client": apiClientSrc,
    },
    dedupe: ["@inflatable-cookie/underlay"],
  },
  optimizeDeps: {
    exclude: [
      "@inflatable-cookie/underlay",
      "@inflatable-cookie/underlay/nightfire",
      "@inflatable-cookie/underlay/runtime",
      "@inflatable-cookie/underlay/styles",
      "@inflatable-cookie/underlay/client",
    ],
  },
  ssr: {
    noExternal: ["bits-ui", "svelte-toolbelt", "lucide-svelte"],
  },
  server: {
    port: 41003,
    allowedHosts: [
      "acme.test",
      "admin.acme.test",
      "api.acme.test",
    ],
    watch: {
      // Always use polling: this codebase runs exclusively inside Docker containers
      // where inotify/fs.events don't propagate from the host filesystem.
      usePolling: true,
      ignored: [
        "!**/node_modules/@inflatable-cookie/underlay/**",
        "!**/node_modules/@acme/api-client/**",
      ],
    },
  },
  test: {
    include: ["src/**/*.{test,spec}.ts"],
    environment: "node",
  },
});

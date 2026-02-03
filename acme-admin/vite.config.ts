import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig, type PluginOption } from "vite";
import devtoolsJson from "vite-plugin-devtools-json";

export default defineConfig({
    plugins: [devtoolsJson(), sveltekit() as PluginOption],
    test: {
        include: ["tests/**/*.{test,spec}.ts"],
        environment: "jsdom",
        globals: true,
        setupFiles: ["tests/setup.ts"],
    },
    resolve: {
        dedupe: ["@decodelabs/underlay"],
    },
    optimizeDeps: {
        // Underlay is a local `file:` dependency and changes frequently.
        exclude: [
            "@decodelabs/underlay",
            "@decodelabs/underlay/components",
            "@decodelabs/underlay/patterns",
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
        port: 40012,
        strictPort: true,
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
        port: 40012,
        strictPort: true,
    },
});

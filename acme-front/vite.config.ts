/// <reference types="node" />

import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig, type PluginOption } from "vite";
import { fileURLToPath } from "node:url";

const apiClientSrc = fileURLToPath(new URL("../acme-client/src", import.meta.url));

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
			}
		},
		sveltekit() as PluginOption
	],
	resolve: {
		alias: {
			"@api-client": apiClientSrc
		},
		dedupe: ["@decodelabs/underlay"]
	},
	optimizeDeps: {
		exclude: [
			"@decodelabs/underlay",
			"@decodelabs/underlay/components",
			"@decodelabs/underlay/nightfire",
			"@decodelabs/underlay/patterns",
			"@decodelabs/underlay/styles",
			"@decodelabs/underlay/client"
		]
	},
	ssr: {
		noExternal: ["bits-ui", "svelte-toolbelt", "lucide-svelte"]
	},
	server: {
		port: 41003,
		watch: {
			ignored: [
				"!**/node_modules/@decodelabs/underlay/**",
				"!**/node_modules/@acme/api-client/**"
			]
		}
	}
});

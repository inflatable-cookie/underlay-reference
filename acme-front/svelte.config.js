import adapter from "@sveltejs/adapter-static";
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),
	kit: {
		// Static adapter for hybrid deployment:
		// - pre-rendered pages where opted-in
		// - SPA fallback for everything else
		adapter: adapter({
			fallback: "200.html",
			strict: true
		}),
		alias: {
			"@app": "src",
			"@api-client": "../acme-client/src",
			"@decodelabs/underlay": "../../underlay/ts/src"
		},
		prerender: {
			handleUnseenRoutes: "ignore"
		}
	}
};

export default config;

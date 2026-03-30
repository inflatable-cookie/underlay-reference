import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	// Consult https://svelte.dev/docs/kit/integrations
	// for more information about preprocessors
	preprocess: vitePreprocess(),

	kit: {
		// Static adapter for SPA deployment - no Node.js server required.
		// All routes fall back to index.html for client-side routing.
		adapter: adapter({
			// SPA mode: all routes handled by client-side router
			fallback: 'index.html',
			// Strict mode ensures no server-side code accidentally slips through
			strict: true
		}),
		alias: {
			"@app": "src",
			"@api-client": "../acme-client/src",
			"acme-client": "../acme-client/src"
		},
		prerender: {
			// Ignore dynamic routes during prerendering - they're handled client-side
			// via the fallback index.html
			handleUnseenRoutes: 'ignore'
		}
	}
};

export default config;

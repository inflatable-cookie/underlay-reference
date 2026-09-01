import { configureAcmeClient } from "@api-client";
import { resolvePublicApiConfig } from "$lib/config/public-api";

const config = resolvePublicApiConfig();

// `@api-client` and `acme-client` are both aliased to packages/acme-client/src in
// svelte.config.js and vite.config.ts, so they resolve to one module with one
// stored config. Configuring it once here is enough.
configureAcmeClient(config);

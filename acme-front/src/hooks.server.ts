import type { Handle } from "@sveltejs/kit";
import { dev } from "$app/environment";
import {
  createCspConfig,
  generateNonce,
  applyCspHeaders,
  createCspResolveOptions,
} from "@decodelabs/underlay/server";
import { env as privateEnv } from "$env/dynamic/private";
import { resolvePublicApiConfig } from "$lib/config/public-api";

import { configureAcmeClient } from "@api-client/utils/client-factory.js";

const apiConfig = resolvePublicApiConfig();

configureAcmeClient({
  baseUrl: apiConfig.baseUrl,
  apiVersion: apiConfig.apiVersion,
});

const cspReportOnly = privateEnv.CSP_REPORT_ONLY
  ? privateEnv.CSP_REPORT_ONLY === "true"
  : dev;

const cspConfig = createCspConfig({
  connectSrc: [apiConfig.baseUrl],
  reportOnly: cspReportOnly,
});

export const handle: Handle = async ({ event, resolve }) => {
  const nonce = generateNonce();

  const response = await resolve(
    event,
    createCspResolveOptions(nonce, {
      filterSerializedResponseHeaders: (name: string) => name === "content-type",
    }),
  );

  if (!dev) {
    applyCspHeaders(response, cspConfig, nonce);
  }

  return response;
};

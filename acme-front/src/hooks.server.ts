import type { Handle } from "@sveltejs/kit";
import { dev } from "$app/environment";
import {
  createCspConfig,
  generateNonce,
  applyCspHeaders,
  createCspResolveOptions,
} from "@decodelabs/underlay/server";
import { env } from "$env/dynamic/public";
import { env as privateEnv } from "$env/dynamic/private";

import { configureAcmeClient } from "@api-client";

configureAcmeClient({
  baseUrl: env.PUBLIC_API_URL ?? "http://localhost:40011",
  apiVersion: env.PUBLIC_API_VERSION ?? "2025-01-01",
});

const cspReportOnly = privateEnv.CSP_REPORT_ONLY
  ? privateEnv.CSP_REPORT_ONLY === "true"
  : dev;

const cspConfig = createCspConfig({
  connectSrc: [env.PUBLIC_API_URL ?? "http://localhost:40011"],
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

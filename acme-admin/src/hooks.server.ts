import {
  applyCspHeaders,
  createCspConfig,
  createCspResolveOptions,
  generateNonce,
} from "@inflatable-cookie/underlay/server";
import { UnderlayHttpError } from "@inflatable-cookie/underlay/client/errors";
import { dev } from "$app/environment";
import type { Handle, HandleServerError } from "@sveltejs/kit";
import { env as privateEnv } from "$env/dynamic/private";

import { configureAcmeClient as configureAliasedAcmeClient } from "@api-client";
import { configureAcmeClient as configurePackageAcmeClient } from "@api-client";
import { resolvePublicApiConfig } from "$lib/config/public-api";

const config = resolvePublicApiConfig();

configureAliasedAcmeClient(config);
configurePackageAcmeClient(config);

const cspReportOnly = privateEnv.CSP_REPORT_ONLY
  ? privateEnv.CSP_REPORT_ONLY === "true"
  : dev;

const cspConfig = createCspConfig({
  connectSrc: [config.baseUrl],
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

  // Apply CSP headers only in production (Vite's HMR needs inline scripts)
  if (!dev) {
    applyCspHeaders(response, cspConfig, nonce);
  }

  return response;
};

export const handleError: HandleServerError = async ({ error: err }) => {
  if (err instanceof UnderlayHttpError) {
    return {
      message: err.message,
      status: err.status,
      code: err.code,
    };
  }

  if (err instanceof Error && "status" in err && "code" in err) {
    const apiError = err as Error & { status: number; code: string };
    return {
      message: apiError.message,
      status: apiError.status,
      code: apiError.code,
    };
  }

  console.error("Unexpected error:", err);
  return { message: "An unexpected error occurred" };
};

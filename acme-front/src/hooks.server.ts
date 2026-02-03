import type { Handle } from "@sveltejs/kit";
import {
  createCspConfig,
  generateNonce,
  applyCspHeaders,
  createCspResolveOptions,
} from "@decodelabs/underlay/server";
import { env } from "$env/dynamic/public";

import { configureAcmeClient } from "@api-client";

configureAcmeClient({
  baseUrl: env.PUBLIC_API_URL ?? "http://localhost:40011",
  apiVersion: env.PUBLIC_API_VERSION ?? "v1",
});

const cspConfig = createCspConfig({
  connectSrc: [env.PUBLIC_API_URL ?? "http://localhost:40011"],
  reportOnly: true,
});

export const handle: Handle = async ({ event, resolve }) => {
  const nonce = generateNonce();

  const response = await resolve(
    event,
    createCspResolveOptions(nonce, {
      filterSerializedResponseHeaders: (name: string) => name === "content-type",
    }),
  );

  applyCspHeaders(response, cspConfig, nonce);

  return response;
};

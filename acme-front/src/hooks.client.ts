import { env } from "$env/dynamic/public";

import { configureAcmeClient } from "@api-client";

configureAcmeClient({
  baseUrl: env.PUBLIC_API_URL ?? "http://localhost:40011",
  apiVersion: env.PUBLIC_API_VERSION ?? "2025-01-01",
});

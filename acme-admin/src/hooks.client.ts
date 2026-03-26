import { env } from "$env/dynamic/public";

import { configureAcmeClient as configureAliasedAcmeClient } from "@api-client";
import { configureAcmeClient as configurePackageAcmeClient } from "acme-client";

const config = {
  baseUrl: env.PUBLIC_API_URL ?? "http://localhost:40011",
  apiVersion: env.PUBLIC_API_VERSION ?? "2025-01-01",
};

// `acme-admin` still has a mixed import graph (`@api-client` and `acme-client`).
// Configure both entrypoints so commands share the same runtime config until the
// app is fully normalized to one import path.
configureAliasedAcmeClient(config);
configurePackageAcmeClient(config);

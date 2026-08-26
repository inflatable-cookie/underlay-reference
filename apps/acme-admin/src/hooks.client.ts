import { configureAcmeClient as configureAliasedAcmeClient } from "@api-client";
import { configureAcmeClient as configurePackageAcmeClient } from "@api-client";
import { resolvePublicApiConfig } from "$lib/config/public-api";

const config = resolvePublicApiConfig();

// `acme-admin` still has a mixed import graph (`@api-client` and `acme-client`).
// Configure both entrypoints so commands share the same runtime config until the
// app is fully normalized to one import path.
configureAliasedAcmeClient(config);
configurePackageAcmeClient(config);

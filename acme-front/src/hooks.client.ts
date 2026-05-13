import { configureAcmeClient } from "@api-client/utils/client-factory.js";
import { resolvePublicApiConfig } from "$lib/config/public-api";

const apiConfig = resolvePublicApiConfig();

configureAcmeClient({
  baseUrl: apiConfig.baseUrl,
  apiVersion: apiConfig.apiVersion,
});

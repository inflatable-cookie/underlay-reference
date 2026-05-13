export { publicApiConfig } from "./public-api.generated";

import { publicApiConfig } from "./public-api.generated";

export function resolvePublicApiConfig(): {
  baseUrl: string;
  apiVersion: string;
} {
  return {
    baseUrl: publicApiConfig.baseUrl,
    apiVersion: publicApiConfig.apiVersion,
  };
}

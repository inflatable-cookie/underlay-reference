import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  loadConfigStack,
  readDottedValue,
  type ConfigTable,
} from "../../../underlay/ts/src/server/config-stack.ts";

const appRoot = fileURLToPath(new URL("..", import.meta.url));
const configRoot = fileURLToPath(new URL("../..", import.meta.url));
const outputPath = join(appRoot, "src/lib/config/public-api.generated.ts");

export function generatePublicApiConfig(): void {
  const config = loadConfigStack({ projectRoot: configRoot });
  const publicApiConfig = {
    baseUrl: requiredString(config, "public_api.base_url"),
    apiVersion: requiredString(config, "public_api.api_version"),
    frontUrl: requiredString(config, "public_api.front_url"),
    adminUrl: requiredString(config, "public_api.admin_url"),
  };

  mkdirSync(dirname(outputPath), { recursive: true });
  writeFileSync(
    outputPath,
    `export const publicApiConfig = ${JSON.stringify(publicApiConfig, null, 2)} as const;\n`,
  );
}

function requiredString(config: ConfigTable, key: string): string {
  const value = readDottedValue(config, key);
  if (typeof value !== "string" || value.trim().length === 0) {
    throw new Error(`Missing string config value: ${key}`);
  }
  return value;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  generatePublicApiConfig();
}

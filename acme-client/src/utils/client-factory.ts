import { HttpClient } from "./http-client.js";

export interface AcmeClientConfig {
  baseUrl: string;
  apiVersion: string;
}

export interface HttpClientOptions {
  fetchFn?: typeof fetch;
  accessToken?: string | null;
  credentials?: RequestCredentials;
  onRefresh?: (fetchFn: typeof fetch) => Promise<string | null>;
}

export type Audience = "admin" | "front" | "shared";

const AUDIENCE_PREFIXES: Record<Audience, string[]> = {
  admin: ["/v1/admin/"],
  front: ["/v1/"],
  shared: ["/v1/health", "/v1/auth/", "/v1/account/"],
};

function validatePath(path: string, audience: Audience): void {
  const allowedPrefixes = AUDIENCE_PREFIXES[audience];
  const isAllowed = allowedPrefixes.some((prefix) => path.startsWith(prefix));

  if (!isAllowed) {
    throw new Error(
      `Path "${path}" is not allowed for audience "${audience}". ` +
        `Allowed prefixes: ${allowedPrefixes.join(", ")}`,
    );
  }
}

interface GuardedHttpClient {
  get<T>(path: string): Promise<T>;
  post<T>(path: string, body: unknown): Promise<T>;
  put<T>(path: string, body: unknown): Promise<T>;
  patch<T>(path: string, body: unknown): Promise<T>;
  delete<T>(path: string): Promise<T>;
}

let storedConfig: AcmeClientConfig | null = null;

export function configureAcmeClient(config: AcmeClientConfig): void {
  storedConfig = config;
}

function getConfig(): AcmeClientConfig {
  if (!storedConfig) {
    throw new Error(
      "acme-client not configured. Call configureAcmeClient() before using commands.",
    );
  }
  return storedConfig;
}

export function getHttpClient(options?: HttpClientOptions): HttpClient {
  const config = getConfig();

  return new HttpClient({
    baseUrl: config.baseUrl,
    apiVersion: config.apiVersion,
    fetchFn: options?.fetchFn,
    getToken: options?.accessToken ? () => options.accessToken! : undefined,
    credentials: options?.credentials,
    onRefresh: options?.onRefresh
      ? () => options.onRefresh!(options.fetchFn ?? fetch)
      : undefined,
  });
}

function createGuardedClient(options: HttpClientOptions, audience: Audience): GuardedHttpClient {
  const baseClient = getHttpClient(options);

  return {
    async get<T>(path: string): Promise<T> {
      validatePath(path, audience);
      return baseClient.get<T>(path);
    },
    async post<T>(path: string, body: unknown): Promise<T> {
      validatePath(path, audience);
      return baseClient.post<T>(path, body);
    },
    async put<T>(path: string, body: unknown): Promise<T> {
      validatePath(path, audience);
      return baseClient.put<T>(path, body);
    },
    async patch<T>(path: string, body: unknown): Promise<T> {
      validatePath(path, audience);
      return baseClient.patch<T>(path, body);
    },
    async delete<T>(path: string): Promise<T> {
      validatePath(path, audience);
      return baseClient.delete<T>(path);
    },
  };
}

export function getAdminHttpClient(options?: HttpClientOptions): GuardedHttpClient {
  return createGuardedClient(options ?? {}, "admin");
}

export function getFrontHttpClient(options?: HttpClientOptions): GuardedHttpClient {
  return createGuardedClient(options ?? {}, "front");
}

export function getSharedHttpClient(options?: HttpClientOptions): GuardedHttpClient {
  return createGuardedClient(options ?? {}, "shared");
}

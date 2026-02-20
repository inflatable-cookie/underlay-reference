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
  /**
   * Whether to enable CSRF protection for mutating requests.
   * Defaults to true when credentials are 'include'.
   */
  enableCsrf?: boolean;
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
  get<T>(path: string, headers?: Record<string, string>): Promise<T>;
  getWithMeta<T>(
    path: string,
    headers?: Record<string, string>
  ): Promise<{ status: number; headers: Record<string, string>; body: T | null }>;
  post<T>(path: string, body: unknown, headers?: Record<string, string>): Promise<T>;
  put<T>(path: string, body: unknown, headers?: Record<string, string>): Promise<T>;
  putWithMeta<T>(
    path: string,
    body: unknown,
    headers?: Record<string, string>
  ): Promise<{ status: number; headers: Record<string, string>; body: T | null }>;
  patch<T>(path: string, body: unknown, headers?: Record<string, string>): Promise<T>;
  patchWithMeta<T>(
    path: string,
    body: unknown,
    headers?: Record<string, string>
  ): Promise<{ status: number; headers: Record<string, string>; body: T | null }>;
  delete<T>(path: string, headers?: Record<string, string>): Promise<T>;
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
    enableCsrf: options?.enableCsrf,
    onRefresh: options?.onRefresh
      ? () => options.onRefresh!(options.fetchFn ?? fetch)
      : undefined,
  });
}

function createGuardedClient(options: HttpClientOptions, audience: Audience): GuardedHttpClient {
  const baseClient = getHttpClient(options);

  return {
    async get<T>(path: string, headers?: Record<string, string>): Promise<T> {
      validatePath(path, audience);
      return baseClient.get<T>(path, headers);
    },
    async getWithMeta<T>(
      path: string,
      headers?: Record<string, string>
    ): Promise<{ status: number; headers: Record<string, string>; body: T | null }> {
      validatePath(path, audience);
      return baseClient.getWithMeta<T>(path, headers);
    },
    async post<T>(path: string, body: unknown, headers?: Record<string, string>): Promise<T> {
      validatePath(path, audience);
      return baseClient.post<T>(path, body, headers);
    },
    async put<T>(path: string, body: unknown, headers?: Record<string, string>): Promise<T> {
      validatePath(path, audience);
      return baseClient.put<T>(path, body, headers);
    },
    async putWithMeta<T>(
      path: string,
      body: unknown,
      headers?: Record<string, string>
    ): Promise<{ status: number; headers: Record<string, string>; body: T | null }> {
      validatePath(path, audience);
      return baseClient.putWithMeta<T>(path, body, headers);
    },
    async patch<T>(path: string, body: unknown, headers?: Record<string, string>): Promise<T> {
      validatePath(path, audience);
      return baseClient.patch<T>(path, body, headers);
    },
    async patchWithMeta<T>(
      path: string,
      body: unknown,
      headers?: Record<string, string>
    ): Promise<{ status: number; headers: Record<string, string>; body: T | null }> {
      validatePath(path, audience);
      return baseClient.patchWithMeta<T>(path, body, headers);
    },
    async delete<T>(path: string, headers?: Record<string, string>): Promise<T> {
      validatePath(path, audience);
      return baseClient.delete<T>(path, headers);
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

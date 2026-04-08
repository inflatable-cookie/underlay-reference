import {
  createHttpClient as createUnderlayHttpClient,
  type HttpClient as UnderlayHttpClient,
  type HttpClientOptions,
  type RefreshContext,
} from "@decodelabs/underlay/client/http";
import {
  UnderlayHttpError,
} from "@decodelabs/underlay/client/errors";

import type { ApiError } from "../types/common-types.js";
import { getCsrfHeaders, clearCsrfToken } from "./csrf-manager.js";

export interface AcmeClientConfig {
  baseUrl: string;
  apiVersion: string;
  getToken?: () => Promise<string | null> | string | null;
  timeoutMs?: number;
  maxRetries?: number;
  fetchFn?: typeof fetch;
  credentials?: RequestCredentials;
  onRefresh?: () => Promise<string | null>;
  /**
   * Whether to automatically inject CSRF tokens for mutating requests.
   * Default: true when credentials are 'include'.
   */
  enableCsrf?: boolean;
}

export interface HttpResponseMeta<T> {
  status: number;
  headers: Record<string, string>;
  body: T | null;
}

function isPlainObject(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && value.constructor === Object;
}

function camelToSnake(key: string): string {
  return key.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase();
}

function snakeToCamel(key: string): string {
  return key.replace(/_([a-z])/g, (_, c: string) => c.toUpperCase());
}

function toSnakeCaseValue<T>(value: T): T {
  if (Array.isArray(value)) {
    return value.map((item) => toSnakeCaseValue(item)) as T;
  }
  if (!isPlainObject(value)) {
    return value;
  }

  const result: Record<string, unknown> = {};
  for (const [key, nestedValue] of Object.entries(value)) {
    result[camelToSnake(key)] = toSnakeCaseValue(nestedValue);
  }
  return result as T;
}

function toCamelCaseValue<T>(value: T): T {
  if (Array.isArray(value)) {
    return value.map((item) => toCamelCaseValue(item)) as T;
  }
  if (!isPlainObject(value)) {
    return value;
  }

  const result: Record<string, unknown> = {};
  for (const [key, nestedValue] of Object.entries(value)) {
    result[snakeToCamel(key)] = toCamelCaseValue(nestedValue);
  }
  return result as T;
}

function convertError(error: unknown): never {
  if (error instanceof UnderlayHttpError) {
    const underlayError = error as UnderlayHttpError;
    const rawError = underlayError.envelope?.error as Record<string, unknown> | undefined;
    const requestId = rawError?.request_id as string | undefined;
    const fieldErrors = rawError?.field_errors as Record<string, unknown> | undefined;

    const apiError: ApiError = Object.assign(new Error(underlayError.message), {
      status: underlayError.status,
      code: underlayError.envelope?.error.code ?? "unknown_error",
      details: fieldErrors
        ? { fieldErrors }
        : undefined,
      requestId,
      raw: underlayError.envelope,
    });

    throw apiError;
  }

  throw error;
}

export class HttpClient {
  private underlayClient: UnderlayHttpClient;
  private fetchFn: typeof fetch;
  private enableCsrf: boolean;

  constructor(config: AcmeClientConfig) {
    const underlayOptions: HttpClientOptions = {
      baseUrl: config.baseUrl.replace(/\/+$/, ""),
      defaultHeaders: {
        "X-Api-Version": config.apiVersion,
      },
      fetch: config.fetchFn,
      timeoutMs: config.timeoutMs ?? 8000,
      maxRetries: config.maxRetries ?? 3,
      retryStatuses: [502, 503, 504],
      auth: config.getToken
        ? {
            getAccessToken: config.getToken,
            refresh: config.onRefresh
              ? async (ctx: RefreshContext) => {
                  const newToken = await config.onRefresh?.();
                  if (newToken) {
                    await ctx.setAccessToken(newToken);
                    return { success: true, accessToken: newToken };
                  }
                  return { success: false };
                }
              : undefined,
          }
        : undefined,
      credentials: config.credentials,
      debug: false,
    };

    this.underlayClient = createUnderlayHttpClient(underlayOptions);
    this.fetchFn = config.fetchFn ?? globalThis.fetch;
    // Enable CSRF by default when using cookies
    this.enableCsrf = config.enableCsrf ?? (config.credentials === "include");
  }

  async get<T>(path: string, headers?: Record<string, string>): Promise<T> {
    try {
      const response = await this.underlayClient.get<unknown>(path, headers);
      return toCamelCaseValue(response) as T;
    } catch (error) {
      convertError(error);
    }
  }

  async getWithMeta<T>(
    path: string,
    headers?: Record<string, string>
  ): Promise<HttpResponseMeta<T>> {
    try {
      const response = await this.underlayClient.getWithMeta<unknown>(path, headers);
      return {
        status: response.status,
        headers: response.headers,
        body: response.body ? (toCamelCaseValue(response.body) as T) : null,
      };
    } catch (error) {
      convertError(error);
    }
  }

  async post<T>(path: string, body: unknown, headers?: Record<string, string>): Promise<T> {
    try {
      // Auto-inject CSRF token for mutating requests when using cookies
      const csrfHeaders = this.enableCsrf ? await getCsrfHeaders(this.fetchFn) : {};
      const mergedHeaders = { ...csrfHeaders, ...headers };
      
      const response = await this.underlayClient.post<unknown>(
        path,
        toSnakeCaseValue(body),
        mergedHeaders,
      );
      return toCamelCaseValue(response) as T;
    } catch (error) {
      // Clear CSRF token on auth errors (session might be invalid)
      if (error instanceof UnderlayHttpError && (error as UnderlayHttpError).status === 401) {
        clearCsrfToken();
      }
      convertError(error);
    }
  }

  async put<T>(path: string, body: unknown, headers?: Record<string, string>): Promise<T> {
    try {
      const csrfHeaders = this.enableCsrf ? await getCsrfHeaders(this.fetchFn) : {};
      const mergedHeaders = { ...csrfHeaders, ...headers };
      
      const response = await this.underlayClient.put<unknown>(
        path,
        toSnakeCaseValue(body),
        mergedHeaders,
      );
      return toCamelCaseValue(response) as T;
    } catch (error) {
      if (error instanceof UnderlayHttpError && (error as UnderlayHttpError).status === 401) {
        clearCsrfToken();
      }
      convertError(error);
    }
  }

  async putWithMeta<T>(
    path: string,
    body: unknown,
    headers?: Record<string, string>
  ): Promise<HttpResponseMeta<T>> {
    try {
      const csrfHeaders = this.enableCsrf ? await getCsrfHeaders(this.fetchFn) : {};
      const mergedHeaders = { ...csrfHeaders, ...headers };

      const response = await this.underlayClient.requestWithMeta<unknown>({
        method: "PUT",
        path,
        body: toSnakeCaseValue(body),
        headers: mergedHeaders,
      });
      return {
        status: response.status,
        headers: response.headers,
        body: response.body ? (toCamelCaseValue(response.body) as T) : null,
      };
    } catch (error) {
      if (error instanceof UnderlayHttpError && (error as UnderlayHttpError).status === 401) {
        clearCsrfToken();
      }
      convertError(error);
    }
  }

  async patch<T>(path: string, body: unknown, headers?: Record<string, string>): Promise<T> {
    try {
      const csrfHeaders = this.enableCsrf ? await getCsrfHeaders(this.fetchFn) : {};
      const mergedHeaders = { ...csrfHeaders, ...headers };
      
      const response = await this.underlayClient.patch<unknown>(
        path,
        toSnakeCaseValue(body),
        mergedHeaders,
      );
      return toCamelCaseValue(response) as T;
    } catch (error) {
      if (error instanceof UnderlayHttpError && (error as UnderlayHttpError).status === 401) {
        clearCsrfToken();
      }
      convertError(error);
    }
  }

  async patchWithMeta<T>(
    path: string,
    body: unknown,
    headers?: Record<string, string>
  ): Promise<HttpResponseMeta<T>> {
    try {
      const csrfHeaders = this.enableCsrf ? await getCsrfHeaders(this.fetchFn) : {};
      const mergedHeaders = { ...csrfHeaders, ...headers };

      const response = await this.underlayClient.requestWithMeta<unknown>({
        method: "PATCH",
        path,
        body: toSnakeCaseValue(body),
        headers: mergedHeaders,
      });
      return {
        status: response.status,
        headers: response.headers,
        body: response.body ? (toCamelCaseValue(response.body) as T) : null,
      };
    } catch (error) {
      if (error instanceof UnderlayHttpError && (error as UnderlayHttpError).status === 401) {
        clearCsrfToken();
      }
      convertError(error);
    }
  }

  async delete<T>(path: string, headers?: Record<string, string>): Promise<T> {
    try {
      const csrfHeaders = this.enableCsrf ? await getCsrfHeaders(this.fetchFn) : {};
      const mergedHeaders = { ...csrfHeaders, ...headers };
      
      const response = await this.underlayClient.delete<unknown>(path, mergedHeaders);
      return toCamelCaseValue(response) as T;
    } catch (error) {
      if (error instanceof UnderlayHttpError && (error as UnderlayHttpError).status === 401) {
        clearCsrfToken();
      }
      convertError(error);
    }
  }
}

export { clearCsrfToken };
export * from "./csrf-manager.js";

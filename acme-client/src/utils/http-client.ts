import {
  createHttpClient as createUnderlayHttpClient,
  type HttpClient as UnderlayHttpClient,
  type HttpClientOptions,
  type RefreshContext,
  UnderlayHttpError,
} from "@decodelabs/underlay/client";

import type { ApiError } from "../types/common-types.js";

export interface AcmeClientConfig {
  baseUrl: string;
  apiVersion: string;
  getToken?: () => Promise<string | null> | string | null;
  timeoutMs?: number;
  maxRetries?: number;
  fetchFn?: typeof fetch;
  credentials?: RequestCredentials;
  onRefresh?: () => Promise<string | null>;
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
    const rawError = error.envelope?.error as Record<string, unknown> | undefined;
    const requestId = rawError?.request_id as string | undefined;
    const fieldErrors = rawError?.field_errors as Record<string, unknown> | undefined;

    const apiError: ApiError = Object.assign(new Error(error.message), {
      status: error.status,
      code: error.envelope?.error.code ?? "unknown_error",
      details: fieldErrors
        ? { fieldErrors }
        : undefined,
      requestId,
      raw: error.envelope,
    });

    throw apiError;
  }

  throw error;
}

export class HttpClient {
  private underlayClient: UnderlayHttpClient;

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
  }

  async get<T>(path: string): Promise<T> {
    try {
      const response = await this.underlayClient.get<unknown>(path);
      return toCamelCaseValue(response) as T;
    } catch (error) {
      convertError(error);
    }
  }

  async post<T>(path: string, body: unknown): Promise<T> {
    try {
      const response = await this.underlayClient.post<unknown>(path, toSnakeCaseValue(body));
      return toCamelCaseValue(response) as T;
    } catch (error) {
      convertError(error);
    }
  }

  async put<T>(path: string, body: unknown): Promise<T> {
    try {
      const response = await this.underlayClient.put<unknown>(path, toSnakeCaseValue(body));
      return toCamelCaseValue(response) as T;
    } catch (error) {
      convertError(error);
    }
  }

  async patch<T>(path: string, body: unknown): Promise<T> {
    try {
      const response = await this.underlayClient.patch<unknown>(path, toSnakeCaseValue(body));
      return toCamelCaseValue(response) as T;
    } catch (error) {
      convertError(error);
    }
  }

  async delete<T>(path: string): Promise<T> {
    try {
      const response = await this.underlayClient.delete<unknown>(path);
      return toCamelCaseValue(response) as T;
    } catch (error) {
      convertError(error);
    }
  }
}

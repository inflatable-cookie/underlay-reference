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

function convertError(error: unknown): never {
  if (error instanceof UnderlayHttpError) {
    const rawError = error.envelope?.error as Record<string, unknown> | undefined;
    const requestId = rawError?.requestId as string | undefined;

    const apiError: ApiError = Object.assign(new Error(error.message), {
      status: error.status,
      code: error.envelope?.error.code ?? "unknown_error",
      details: error.envelope?.error.fieldErrors
        ? { fieldErrors: error.envelope.error.fieldErrors }
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
      return await this.underlayClient.get<T>(path);
    } catch (error) {
      convertError(error);
    }
  }

  async post<T>(path: string, body: unknown): Promise<T> {
    try {
      return await this.underlayClient.post<T>(path, body);
    } catch (error) {
      convertError(error);
    }
  }

  async put<T>(path: string, body: unknown): Promise<T> {
    try {
      return await this.underlayClient.put<T>(path, body);
    } catch (error) {
      convertError(error);
    }
  }

  async patch<T>(path: string, body: unknown): Promise<T> {
    try {
      return await this.underlayClient.patch<T>(path, body);
    } catch (error) {
      convertError(error);
    }
  }

  async delete<T>(path: string): Promise<T> {
    try {
      return await this.underlayClient.delete<T>(path);
    } catch (error) {
      convertError(error);
    }
  }
}

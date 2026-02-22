declare module "$app/environment" {
  export const browser: boolean;
  export const dev: boolean;
  export const building: boolean;
  export const version: string;
}

declare module "$app/stores" {
  import type { Readable } from "svelte/store";

  export const page: Readable<{
    url: URL;
    params: Record<string, string>;
    route: { id: string | null };
    status: number;
    error: unknown;
    data: Record<string, unknown>;
    form: unknown;
    state: Record<string, unknown>;
  }>;
}

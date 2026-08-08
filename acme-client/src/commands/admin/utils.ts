import type { QueryParams } from "@inflatable-cookie/underlay/client/query";

export interface WithEtag<T> {
  data: T;
  etag: string | null;
}

export function getHeaderValueCaseInsensitive(
  headers: Record<string, string> | undefined,
  name: string
): string | null {
  if (!headers) return null;
  const target = name.toLowerCase();
  for (const [key, value] of Object.entries(headers)) {
    if (key.toLowerCase() === target) {
      return value;
    }
  }
  return null;
}

export function camelToSnake(value: string): string {
  return value.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase();
}

export function toSnakeQueryParams(query?: QueryParams): QueryParams {
  if (!query) {
    return {};
  }

  return {
    ...query,
    sort: query.sort?.map((item: NonNullable<QueryParams["sort"]>[number]) => ({
      ...item,
      field: camelToSnake(item.field),
    })),
    filters: query.filters?.map((item: NonNullable<QueryParams["filters"]>[number]) => ({
      ...item,
      field: camelToSnake(item.field),
    })),
  };
}

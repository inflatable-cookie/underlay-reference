import type { QueryParams } from "@decodelabs/underlay/client";

export function camelToSnake(value: string): string {
  return value.replace(/([a-z0-9])([A-Z])/g, "$1_$2").toLowerCase();
}

export function toSnakeQueryParams(query?: QueryParams): QueryParams {
  if (!query) {
    return {};
  }

  return {
    ...query,
    sort: query.sort?.map((item) => ({
      ...item,
      field: camelToSnake(item.field),
    })),
    filters: query.filters?.map((item) => ({
      ...item,
      field: camelToSnake(item.field),
    })),
  };
}

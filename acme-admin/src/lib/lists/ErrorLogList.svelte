<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { buildQueryString, parseQueryParams, type QueryParams } from "@decodelabs/underlay/client/query";
  import {
    ErrorLogListPage,
    type ErrorLogDetailLoader,
    type ErrorLogListLoader,
    type ErrorLogStatsLoader,
    toPagedListResult,
  } from "@decodelabs/underlay/templates";
  import { adminCommands } from "@api-client";

  const currentQuery = $derived(parseQueryParams($page.url.searchParams));

  function updateUrl(nextQuery: QueryParams) {
    const url = new URL($page.url);
    url.search = buildQueryString(nextQuery);
    goto(url.toString(), { replaceState: true, keepFocus: true });
  }

  const loadList: ErrorLogListLoader = async (fetchFn, token, request) => {
    const response = await adminCommands.listErrorLogs(fetchFn, token, {
      status_code: request.statusCode,
      limit: request.limit,
      offset: request.offset
    });
    return toPagedListResult(response);
  };

  const loadDetail: ErrorLogDetailLoader = async (id, fetchFn, token) => {
    return await adminCommands.getErrorLog(id, fetchFn, token);
  };

  const loadStats: ErrorLogStatsLoader = async (fetchFn, token) => {
    return await adminCommands.getErrorLogStats(fetchFn, token);
  };
</script>

<ErrorLogListPage
  query={currentQuery}
  onQueryChange={updateUrl}
  {loadList}
  {loadDetail}
  {loadStats}
/>

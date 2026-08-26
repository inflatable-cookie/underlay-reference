<script lang="ts">
  import { createEntityListState } from "@inflatable-cookie/underlay/patterns";
  import {
    ErrorLogListPage,
    type ErrorLogDetailLoader,
    type ErrorLogListLoader,
    type ErrorLogStatsLoader,
    toPagedListResult,
  } from "@inflatable-cookie/underlay/templates";
  import { adminCommands } from "@api-client";

  const listState = createEntityListState({
    queryMode: () => "url",
    title: () => "Error log"
  });

  const loadList: ErrorLogListLoader = async (fetchFn, token, request) => {
    const response = await adminCommands.listErrorLogs(fetchFn, token, {
      status_class: request.statusClass,
      status_code: request.statusCode,
      limit: request.limit,
      page: request.page
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
  query={listState.query}
  onQueryChange={listState.setQuery}
  {loadList}
  {loadDetail}
  {loadStats}
/>

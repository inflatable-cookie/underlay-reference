<script lang="ts">
  import type { QueryParams } from "@inflatable-cookie/underlay/client/query";
  import {
    toPagedListResult,
    UsersListPage,
    type UsersListLoader
  } from "@inflatable-cookie/underlay/templates";
  import { adminCommands, type User } from "@api-client";

  interface Props {
    title?: string;
    hideTitle?: boolean;
    subtitle?: string;
    eyebrow?: string;
    headerLevel?: 1 | 2 | 3 | 4 | 5 | 6;
    backHref?: string;
    backLabel?: string;
    query: QueryParams;
    onQueryChange: (query: QueryParams) => void;
  }

  let {
    title = "Users",
    hideTitle = false,
    subtitle,
    eyebrow,
    headerLevel = 2,
    backHref = "/",
    backLabel = "Back to dashboard",
    query,
    onQueryChange
  }: Props = $props();

  const dataLoader: UsersListLoader<User> = async (fetchFn, token, nextQuery) => {
    if (!token) throw new Error("Not authenticated");
    const response = await adminCommands.listUsers(fetchFn, token, nextQuery);
    const result = toPagedListResult(response);
    return { data: result.data, total: result.total ?? 0, hasMore: result.hasMore };
  };
</script>

<UsersListPage
  {title}
  {hideTitle}
  {subtitle}
  {eyebrow}
  {headerLevel}
  {backHref}
  {backLabel}
  {query}
  {onQueryChange}
  {dataLoader}
  usersBaseHref="/users"
/>

// App-local list wrappers over Underlay list templates.
//
// Two shapes coexist on purpose:
// - `<Resource>List` (UsersList, CategoriesList, MediaList, ...): route-mounted
//   browse wrappers. Query state is route-owned — the route creates
//   `createPageListQueryState({ mode: "url" })` and passes `query` /
//   `onQueryChange` (required props). New sections should follow this shape.
// - `<Resource>ListPage` (ProjectsListPage, TasksListPage): dual-purpose
//   wrappers that also embed into detail pages (category detail, project
//   detail) with parent scope props and a lower `headerLevel`. Their `query` /
//   `onQueryChange` props are optional so embedded mounts keep local query
//   state instead of syncing to the URL.
// - `LabelsList`: dual-purpose wrapper over `createEntityListState` — query
//   state is wrapper-owned and switches between `url` (route mount) and
//   `local` (detail-tab mount) via the `queryMode` prop.
// UserSessionsTab/UserActivityTab are detail-tab wrappers over the Level-2
// `UserSessionsList`/`UserActivityList` template sections.

export { default as AuditLogList } from "./AuditLogList.svelte";
export { default as CategoriesList } from "./CategoriesList.svelte";
export { default as ErrorLogList } from "./ErrorLogList.svelte";
export { default as JobsList } from "./JobsList.svelte";
export { default as LabelsList } from "./LabelsList.svelte";
export { default as MediaList } from "./MediaList.svelte";
export { default as MediaTrashList } from "./MediaTrashList.svelte";
export { default as ProjectsListPage } from "./ProjectsListPage.svelte";
export { default as ScheduledTasksList } from "./ScheduledTasksList.svelte";
export { default as TasksListPage } from "./TasksListPage.svelte";
export { default as UserActivityTab } from "./UserActivityTab.svelte";
export { default as UserSessionsTab } from "./UserSessionsTab.svelte";
export { default as UsersList } from "./UsersList.svelte";

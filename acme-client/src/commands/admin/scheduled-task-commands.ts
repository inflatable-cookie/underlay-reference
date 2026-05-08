import type { PagedListResponse, SingleResponse } from "../../types/common-types.js";
import type {
  ScheduledTaskSummary,
  ScheduledTaskDetail,
  ListScheduledTasksQuery,
  ScheduledTaskListResponse,
  TriggerScheduledTaskResult,
} from "../../types/admin-types.js";
import { getAdminHttpClient } from "../../utils/client-factory.js";

export async function listScheduledTasks(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListScheduledTasksQuery
): Promise<ScheduledTaskListResponse> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  const params = new URLSearchParams();
  if (query?.enabled !== undefined) params.set("enabled", String(query.enabled));
  if (query?.page !== undefined) params.set("page", String(query.page));
  if (query?.limit !== undefined) params.set("limit", String(query.limit));

  const queryString = params.toString();
  const path = queryString ? `/v1/admin/scheduled-tasks?${queryString}` : "/v1/admin/scheduled-tasks";

  const response = await http.get<PagedListResponse<ScheduledTaskSummary>>(path);
  return {
    data: response.data,
    total: response.total,
    hasMore: response.hasMore ?? false
  };
}

export async function getScheduledTask(
  taskId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<ScheduledTaskDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<ScheduledTaskDetail>>(
    `/v1/admin/scheduled-tasks/${encodeURIComponent(taskId)}`
  );
  return response.data;
}

export async function toggleScheduledTask(
  taskId: string,
  enabled: boolean,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<void> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  await http.post(
    `/v1/admin/scheduled-tasks/${encodeURIComponent(taskId)}/toggle`,
    { enabled }
  );
}

export async function triggerScheduledTask(
  taskId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<TriggerScheduledTaskResult> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<TriggerScheduledTaskResult>>(
    `/v1/admin/scheduled-tasks/${encodeURIComponent(taskId)}/trigger`,
    {}
  );
  return response.data;
}

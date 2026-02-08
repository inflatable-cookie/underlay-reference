import type { ListResponse, SingleResponse } from "../../types/common-types.js";
import type {
  JobSummary,
  JobDetail,
  JobStats,
  ListJobsQuery,
} from "../../types/admin-types.js";
import { getAdminHttpClient } from "../../utils/client-factory.js";

/**
 * List background jobs.
 *
 * Supports filtering by status and job type.
 */
export async function listJobs(
  fetchFn: typeof fetch,
  accessToken: string,
  query?: ListJobsQuery
): Promise<JobSummary[]> {
  const http = getAdminHttpClient({ fetchFn, accessToken });

  const params = new URLSearchParams();
  if (query?.status) params.set("status", query.status);
  if (query?.jobType) params.set("job_type", query.jobType);
  if (query?.limit !== undefined) params.set("limit", String(query.limit));

  const queryString = params.toString();
  const path = queryString ? `/v1/admin/jobs?${queryString}` : "/v1/admin/jobs";

  const response = await http.get<ListResponse<JobSummary>>(path);
  return response.data;
}

/**
 * Get details of a specific job.
 */
export async function getJob(
  jobId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<JobDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<JobDetail>>(
    `/v1/admin/jobs/${encodeURIComponent(jobId)}`
  );
  return response.data;
}

/**
 * Get job queue statistics.
 */
export async function getJobStats(
  fetchFn: typeof fetch,
  accessToken: string
): Promise<JobStats> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<JobStats>>(
    "/v1/admin/jobs/stats"
  );
  return response.data;
}

/**
 * Cancel a pending or running job.
 */
export async function cancelJob(
  jobId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<JobDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<JobDetail>>(
    `/v1/admin/jobs/${encodeURIComponent(jobId)}/cancel`,
    {}
  );
  return response.data;
}

/**
 * Retry a failed or cancelled job.
 *
 * Creates a new job with the same payload.
 */
export async function retryJob(
  jobId: string,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<JobDetail> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.post<SingleResponse<JobDetail>>(
    `/v1/admin/jobs/${encodeURIComponent(jobId)}/retry`,
    {}
  );
  return response.data;
}

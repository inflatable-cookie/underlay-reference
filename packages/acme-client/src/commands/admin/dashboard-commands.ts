import type { SingleResponse } from "../../types/common-types.js";
import type { DashboardStats } from "../../types/admin-types.js";
import { getAdminHttpClient } from "../../utils/client-factory.js";

/**
 * Get dashboard statistics.
 */
export async function getDashboardStats(
  fetchFn: typeof fetch,
  accessToken: string
): Promise<DashboardStats> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  const response = await http.get<SingleResponse<DashboardStats>>(
    "/v1/admin/dashboard/stats"
  );
  return response.data;
}

import type { SingleResponse } from "@decodelabs/underlay/client/types";

import { getSharedHttpClient } from "../utils/client-factory.js";

export interface HealthData {
  status: string;
}

export async function health(fetchFn: typeof fetch): Promise<HealthData> {
  const http = getSharedHttpClient({ fetchFn });
  const res = await http.get<SingleResponse<HealthData>>("/v1/health");
  return res.data;
}

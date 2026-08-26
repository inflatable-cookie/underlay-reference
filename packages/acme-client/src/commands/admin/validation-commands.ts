import type {
  ValidateFieldPayload,
  ValidationResult,
} from "../../types/admin-types.js";
import { getAdminHttpClient } from "../../utils/client-factory.js";

/**
 * Validate a field value (async form validation).
 *
 * Used for checking uniqueness of slugs, names, etc. before form submission.
 *
 * @example
 * ```typescript
 * const result = await validateField(
 *   { entity: 'category', field: 'slug', value: 'my-category' },
 *   fetch,
 *   accessToken
 * );
 * if (!result.valid) {
 *   showError(result.message);
 * }
 * ```
 */
export async function validateField(
  payload: ValidateFieldPayload,
  fetchFn: typeof fetch,
  accessToken: string
): Promise<ValidationResult> {
  const http = getAdminHttpClient({ fetchFn, accessToken });
  return await http.post<ValidationResult>(
    "/v1/admin/validate-field",
    payload
  );
}

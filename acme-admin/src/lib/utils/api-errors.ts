/**
 * API error handling utilities.
 *
 * Extracts error messages and field errors from API responses.
 */
import { toUserMessage } from "acme-client";

interface ExtractedError {
  message: string;
  fieldErrors?: Record<string, string>;
}

export function isPreconditionFailed(error: unknown): boolean {
  if (!error || typeof error !== "object") return false;
  const maybeStatus = "status" in error ? (error as { status?: unknown }).status : undefined;
  const maybeCode = "code" in error ? (error as { code?: unknown }).code : undefined;
  return maybeStatus === 412 || maybeCode === "resource.precondition_failed";
}

/**
 * Extract user-friendly error message from an API error.
 *
 * @param error - The caught error object
 * @param fallback - Default message if extraction fails
 * @returns Extracted error information
 */
export function extractApiError(error: unknown, fallback: string): ExtractedError {
  const message = toUserMessage(error) ?? fallback;

  // Try to extract field errors if available
  let fieldErrors: Record<string, string> | undefined;
  if (error && typeof error === "object" && "details" in error) {
    const details = (error as { details?: unknown }).details;
    if (details && typeof details === "object" && !Array.isArray(details)) {
      fieldErrors = {};
      for (const [key, value] of Object.entries(details)) {
        if (typeof value === "string") {
          fieldErrors[key] = value;
        }
      }
      if (Object.keys(fieldErrors).length === 0) {
        fieldErrors = undefined;
      }
    }
  }

  return { message, fieldErrors };
}

/**
 * Unit tests for utility functions.
 *
 * These tests demonstrate patterns for testing pure functions
 * without complex mocking requirements.
 */

import { describe, it, expect } from "vitest";
import { extractApiError } from "$lib/utils/api-errors";

describe("extractApiError", () => {
	it("returns a message for null/undefined errors", () => {
		const result = extractApiError(null, "Something went wrong");
		// toUserMessage provides a generic message when error is null
		expect(result.message).toBeTruthy();
		expect(result.fieldErrors).toBeUndefined();
	});

	it("returns fallback for non-object errors", () => {
		const result = extractApiError("string error", "Fallback");
		// toUserMessage handles string errors
		expect(result.message).toBeTruthy();
	});

	it("extracts field errors from details object", () => {
		const error = {
			message: "Validation failed",
			details: {
				email: "Invalid email format",
				password: "Password too short",
			},
		};

		const result = extractApiError(error, "Fallback");
		expect(result.fieldErrors).toEqual({
			email: "Invalid email format",
			password: "Password too short",
		});
	});

	it("ignores non-string field error values", () => {
		const error = {
			message: "Validation failed",
			details: {
				email: "Invalid email",
				count: 42, // Not a string
				nested: { foo: "bar" }, // Not a string
			},
		};

		const result = extractApiError(error, "Fallback");
		expect(result.fieldErrors).toEqual({
			email: "Invalid email",
		});
	});

	it("returns undefined fieldErrors when details is empty", () => {
		const error = {
			message: "Error",
			details: {},
		};

		const result = extractApiError(error, "Fallback");
		expect(result.fieldErrors).toBeUndefined();
	});

	it("handles details as array (ignores it)", () => {
		const error = {
			message: "Error",
			details: ["item1", "item2"],
		};

		const result = extractApiError(error, "Fallback");
		expect(result.fieldErrors).toBeUndefined();
	});
});

// ============================================================================
// Example: Testing formatters and transformers
// ============================================================================

describe("formatters (example patterns)", () => {
	// Example: Testing a relative time formatter
	const formatRelativeTime = (dateStr: string): string => {
		const date = new Date(dateStr);
		const now = new Date();
		const diff = now.getTime() - date.getTime();
		const seconds = Math.floor(diff / 1000);
		const minutes = Math.floor(seconds / 60);
		const hours = Math.floor(minutes / 60);
		const days = Math.floor(hours / 24);

		if (days > 0) return `${days}d ago`;
		if (hours > 0) return `${hours}h ago`;
		if (minutes > 0) return `${minutes}m ago`;
		return "just now";
	};

	it("formats days ago", () => {
		const threeDaysAgo = new Date(Date.now() - 3 * 24 * 60 * 60 * 1000);
		expect(formatRelativeTime(threeDaysAgo.toISOString())).toBe("3d ago");
	});

	it("formats hours ago", () => {
		const twoHoursAgo = new Date(Date.now() - 2 * 60 * 60 * 1000);
		expect(formatRelativeTime(twoHoursAgo.toISOString())).toBe("2h ago");
	});

	it("formats minutes ago", () => {
		const fiveMinutesAgo = new Date(Date.now() - 5 * 60 * 1000);
		expect(formatRelativeTime(fiveMinutesAgo.toISOString())).toBe("5m ago");
	});

	it("formats just now for very recent times", () => {
		const now = new Date();
		expect(formatRelativeTime(now.toISOString())).toBe("just now");
	});
});

// ============================================================================
// Example: Testing validation functions
// ============================================================================

describe("validation patterns", () => {
	// Example email validator
	const isValidEmail = (email: string): boolean => {
		return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
	};

	it("validates correct emails", () => {
		expect(isValidEmail("user@example.com")).toBe(true);
		expect(isValidEmail("user.name@domain.co.uk")).toBe(true);
		expect(isValidEmail("user+tag@example.org")).toBe(true);
	});

	it("rejects invalid emails", () => {
		expect(isValidEmail("not-an-email")).toBe(false);
		expect(isValidEmail("missing@domain")).toBe(false);
		expect(isValidEmail("spaces not@allowed.com")).toBe(false);
		expect(isValidEmail("")).toBe(false);
	});

	// Example status validation
	const isValidStatus = (status: string): boolean => {
		const validStatuses = ["pending", "in_progress", "completed", "cancelled"];
		return validStatuses.includes(status);
	};

	it("validates known statuses", () => {
		expect(isValidStatus("pending")).toBe(true);
		expect(isValidStatus("in_progress")).toBe(true);
		expect(isValidStatus("completed")).toBe(true);
		expect(isValidStatus("cancelled")).toBe(true);
	});

	it("rejects unknown statuses", () => {
		expect(isValidStatus("unknown")).toBe(false);
		expect(isValidStatus("PENDING")).toBe(false); // Case sensitive
		expect(isValidStatus("")).toBe(false);
	});
});

// ============================================================================
// Example: Testing data transformers
// ============================================================================

describe("data transformer patterns", () => {
	interface ApiUser {
		id: string;
		email: string;
		displayName: string | null;
		createdAt: string;
	}

	interface DisplayUser {
		id: string;
		email: string;
		name: string;
		joinedDate: Date;
	}

	// Transform API response to display format
	const toDisplayUser = (apiUser: ApiUser): DisplayUser => ({
		id: apiUser.id,
		email: apiUser.email,
		name: apiUser.displayName ?? apiUser.email.split("@")[0],
		joinedDate: new Date(apiUser.createdAt),
	});

	it("transforms API user to display format", () => {
		const apiUser: ApiUser = {
			id: "123",
			email: "john@example.com",
			displayName: "John Doe",
			createdAt: "2024-01-15T10:30:00Z",
		};

		const result = toDisplayUser(apiUser);

		expect(result.id).toBe("123");
		expect(result.email).toBe("john@example.com");
		expect(result.name).toBe("John Doe");
		expect(result.joinedDate).toBeInstanceOf(Date);
	});

	it("uses email prefix when displayName is null", () => {
		const apiUser: ApiUser = {
			id: "456",
			email: "jane.doe@example.com",
			displayName: null,
			createdAt: "2024-01-15T10:30:00Z",
		};

		const result = toDisplayUser(apiUser);

		expect(result.name).toBe("jane.doe");
	});
});

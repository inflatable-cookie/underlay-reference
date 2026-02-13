import { get } from "svelte/store";
import { beforeEach, describe, expect, it, vi } from "vitest";

type LoginUser = {
	userId: string;
	email: string;
	displayName: string;
	roles: string[];
};

type LoginResponse = {
	sessionId: string;
	accessToken: string;
	user: LoginUser;
};

const gotoMock = vi.fn(async () => undefined);

function createManager(overrides: Partial<Record<string, unknown>> = {}) {
	const user: LoginUser = {
		userId: "user_1",
		email: "user@example.com",
		displayName: "Example User",
		roles: ["user"]
	};

	return {
		initialize: vi.fn(async () => null),
		login: vi.fn(async () => user),
		register: vi.fn(async () => user),
		logout: vi.fn(async () => undefined),
		getToken: vi.fn(() => "access_123"),
		setSession: vi.fn((response: LoginResponse) => response.user),
		refresh: vi.fn(async () => user),
		...overrides
	};
}

async function loadStore(overrides: Partial<Record<string, unknown>> = {}) {
	const manager = createManager(overrides);

	vi.doMock("$app/navigation", () => ({
		goto: gotoMock
	}));
	vi.doMock("$app/environment", () => ({
		browser: true
	}));
	vi.doMock("@api-client", () => ({
		createAuthManager: vi.fn(() => manager)
	}));

	const module = await import("./auth");

	return {
		auth: module.auth,
		currentUser: module.currentUser,
		authLoading: module.authLoading,
		manager
	};
}

describe("auth store", () => {
	beforeEach(() => {
		vi.resetModules();
		vi.clearAllMocks();
	});

	it("sets the user after successful login", async () => {
		const { auth, currentUser, authLoading, manager } = await loadStore();

		await auth.login("user@example.com", "hunter2");

		expect(manager.login).toHaveBeenCalledWith(
			{ email: "user@example.com", password: "hunter2" },
			expect.any(Function)
		);
		expect(get(currentUser)?.email).toBe("user@example.com");
		expect(get(authLoading)).toBe(false);
	});

	it("clears the user and redirects on logout", async () => {
		const { auth, currentUser, manager } = await loadStore();

		auth.setSession({
			sessionId: "session_1",
			accessToken: "access_123",
			user: {
				userId: "user_1",
				email: "user@example.com",
				displayName: "Example User",
				roles: ["user"]
			}
		});

		await auth.logout();

		expect(manager.logout).toHaveBeenCalledWith(expect.any(Function));
		expect(get(currentUser)).toBeNull();
		expect(gotoMock).toHaveBeenCalledWith("/login");
	});
});

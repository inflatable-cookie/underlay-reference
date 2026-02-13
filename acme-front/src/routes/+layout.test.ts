import { describe, expect, it } from "vitest";

import { prerender, ssr } from "./+layout";

describe("root layout config", () => {
	it("keeps SPA defaults for the public site", () => {
		expect(ssr).toBe(false);
		expect(prerender).toBe(true);
	});
});

import { describe, expect, it } from "vitest";

import { load } from "../../../../../src/routes/(app)/projects/[projectId]/+page";

describe("project page load", () => {
	it("returns the projectId from route params", () => {
		const data = load({
			params: {
				projectId: "proj_123"
			}
		} as never);

		expect(data).toEqual({
			projectId: "proj_123"
		});
	});
});

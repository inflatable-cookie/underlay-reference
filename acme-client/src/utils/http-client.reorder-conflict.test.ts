import { beforeEach, describe, expect, it, vi } from "vitest";
import { extractReorderConflict } from "../../../underlay/ts/src/patterns/reorder-conflict";
import { HttpClient } from "./http-client";

const requestMock = vi.fn();

vi.mock("@decodelabs/underlay/client", () => {
  class MockUnderlayHttpError extends Error {
    readonly status: number;
    readonly envelope?: unknown;

    constructor(status: number, message: string, envelope?: unknown) {
      super(message);
      this.name = "UnderlayHttpError";
      this.status = status;
      this.envelope = envelope;
    }
  }

  return {
    UnderlayHttpError: MockUnderlayHttpError,
    createHttpClient: vi.fn(() => ({
      request: requestMock,
      requestWithMeta: requestMock,
      get: requestMock,
      getWithMeta: requestMock,
      post: requestMock,
      put: requestMock,
      patch: requestMock,
      delete: requestMock
    }))
  };
});

const { UnderlayHttpError } = await import("@decodelabs/underlay/client");

describe("HttpClient reorder conflict envelope passthrough", () => {
  beforeEach(() => {
    requestMock.mockReset();
  });

  it("preserves raw.context for reorder conflict extraction", async () => {
    requestMock.mockRejectedValueOnce(
      new UnderlayHttpError(409, "Items have changed since you started reordering.", {
        error: {
          code: "projects.reorder_conflict",
          message: "Items have changed since you started reordering."
        },
        context: {
          added_ids: ["proj_added"],
          removed_ids: ["proj_removed"]
        }
      } as any)
    );

    const client = new HttpClient({
      baseUrl: "http://localhost:4000",
      apiVersion: "v1",
      enableCsrf: false
    });

    try {
      await client.put("/v1/admin/projects/reorder", { ids: [] });
      throw new Error("expected request to fail");
    } catch (error) {
      expect(error).toMatchObject({ status: 409 });
      const conflict = extractReorderConflict(error);
      expect(conflict).toEqual({
        addedIds: ["proj_added"],
        removedIds: ["proj_removed"],
        message: "Items have changed since you started reordering."
      });
    }
  });
});

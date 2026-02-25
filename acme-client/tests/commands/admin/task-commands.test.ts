import { beforeEach, describe, expect, it, vi } from "vitest";

const postMock = vi.fn();

vi.mock("../../../src/utils/client-factory.js", () => ({
  getAdminHttpClient: () => ({
    post: postMock,
  }),
}));

import {
  batchDeleteTasks,
  batchUpdateTaskStatus,
} from "../../../src/commands/admin/task-commands.js";

describe("admin task command endpoints", () => {
  beforeEach(() => {
    postMock.mockReset();
  });

  it("uses /tasks/batch-delete path for batch deletion", async () => {
    postMock.mockResolvedValue({ ok: true, deleted: 2 });

    const result = await batchDeleteTasks(
      "proj 123",
      { ids: ["task_1", "task_2"] },
      fetch,
      "access_token"
    );

    expect(postMock).toHaveBeenCalledWith(
      "/v1/admin/projects/proj%20123/tasks/batch-delete",
      { ids: ["task_1", "task_2"] }
    );
    expect(result).toEqual({ ok: true, deleted: 2 });
  });

  it("uses /tasks/batch-update path for batch status updates", async () => {
    postMock.mockResolvedValue({ ok: true, updated: 3 });

    const result = await batchUpdateTaskStatus(
      "proj 123",
      {
        ids: ["task_1", "task_2", "task_3"],
        status: "completed",
      },
      fetch,
      "access_token"
    );

    expect(postMock).toHaveBeenCalledWith(
      "/v1/admin/projects/proj%20123/tasks/batch-update",
      {
        ids: ["task_1", "task_2", "task_3"],
        status: "completed",
      }
    );
    expect(result).toEqual({ ok: true, updated: 3 });
  });
});
